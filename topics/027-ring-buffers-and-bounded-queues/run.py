#!/usr/bin/env python3
"""Ten mirrored rotation blocks, five candidates and paired deque A/A controls."""
import hashlib,json,os,pathlib,platform,statistics,subprocess,time
root=pathlib.Path(__file__).resolve().parent;os.chdir(root)
out=root/'raw';out.mkdir(exist_ok=True)
def command(args):
    p=subprocess.run(args,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=90)
    if p.returncode:
        (out/('failure-'+str(time.time_ns())+'.txt')).write_text(repr(args)+'\n'+p.stdout)
        raise RuntimeError(repr(args))
    return p.stdout
files=['src/lib.rs','benches/portfolio.rs','run.py','examples/wrapped.rs','tests/contracts.rs']
meta={'hostname':platform.node(),'uname':list(platform.uname()),'cpu':command(['lscpu']),'available_cpus':len(os.sched_getaffinity(0)),'affinity':sorted(os.sched_getaffinity(0)),'rustc':command(['rustc','-Vv']),'features':command(['rustc','--print','cfg','-C','target-cpu=native']),'flags':'--edition 2024 -O -C target-cpu=native','started_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'files':{p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()for p in files}}
(out/'identity.json').write_text(json.dumps(meta,indent=2))
(out/'tests.txt').write_text(command(['rustc','--edition','2024','--test','src/lib.rs','-o','tests-bin'])+command(['./tests-bin']))
command(['rustc','--edition','2024','-O','-C','target-cpu=native','--crate-name','algorithms_ring','--crate-type','rlib','src/lib.rs','-o','libalgorithms_ring.rlib'])
(out/'external-tests.txt').write_text(command(['rustc','--edition','2024','--test','tests/contracts.rs','--extern','algorithms_ring=libalgorithms_ring.rlib','-o','contracts-bin'])+command(['./contracts-bin']))
command(['rustc','--edition','2024','-O','-C','target-cpu=native','benches/portfolio.rs','--extern','algorithms_ring=libalgorithms_ring.rlib','-o','portfolio'])
command(['rustc','--edition','2024','examples/wrapped.rs','--extern','algorithms_ring=libalgorithms_ring.rlib','-o','example']);(out/'example.txt').write_text(command(['./example']))
meta['binary_sha256']=hashlib.sha256(pathlib.Path('portfolio').read_bytes()).hexdigest();(out/'identity.json').write_text(json.dumps(meta,indent=2))
cases=['tiny','steady_power2','steady_odd','burst','large','full','lifecycle'];names=['shift','deque','modulo','branch','mask'];runs=[]
for block in range(10):
    for case in (cases if block%2==0 else cases[::-1]):
        offset=block%5;order=names[offset:]+names[:offset]
        if block>=5:order=order[::-1]
        for pos,name in enumerate(order):
            raw=command(['./portfolio',name,case]);(out/f'{block}-{case}-{pos}-{name}.json').write_text(raw)
            item=json.loads(raw);item.update(block=block,position=pos,label=name);runs.append(item)
        for label in (['A','B']if block%2==0 else ['B','A']):
            raw=command(['./portfolio','deque',case]);(out/f'{block}-{case}-control-{label}.json').write_text(raw)
            item=json.loads(raw);item.update(block=block,label=label);runs.append(item)
    print('completed block',block,flush=True)
summary={}
for case in cases:
    cells={}
    for name in names:
        samples=[r['ns_per_op']for r in runs if r['case']==case and r['label']==name]
        cells[name]={'median':statistics.median(samples),'min':min(samples),'max':max(samples),'samples':samples}
    best=min(names,key=lambda n:cells[n]['median']);rivals=[];ratios={}
    for other in names:
        if other==best:continue
        pair=[next(r['ns_per_op']for r in runs if r['case']==case and r['label']==other and r['block']==b)/next(r['ns_per_op']for r in runs if r['case']==case and r['label']==best and r['block']==b)for b in range(10)]
        ratios[other]=pair
        if min(pair)<=1.03:rivals.append(other)
    aa=[next(r['ns_per_op']for r in runs if r['case']==case and r['label']=='A' and r['block']==b)/next(r['ns_per_op']for r in runs if r['case']==case and r['label']=='B' and r['block']==b)for b in range(10)]
    checks={r['checksum']for r in runs if r['case']==case};assert len(checks)==1
    summary[case]={'candidates':cells,'lowest_median':best,'selection':best if not rivals else 'unresolved: '+best+' / '+','.join(rivals),'paired_ratios':ratios,'AA_ratios':aa,'checksum':checks.pop()}
assert len(runs)==490
for path,digest in meta['files'].items():assert hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()==digest
assert hashlib.sha256(pathlib.Path('portfolio').read_bytes()).hexdigest()==meta['binary_sha256']
(out/'summary.json').write_text(json.dumps(summary,indent=2))
try:(out/'linked-disassembly.txt').write_text(command(['objdump','-dC','portfolio']))
except (OSError,RuntimeError) as e:(out/'codegen-unavailable.txt').write_text(str(e))
print(json.dumps({k:v['selection']for k,v in summary.items()}))
