#!/usr/bin/env python3
"""Fixed mirrored process schedule. Run from the source root; retain every output."""
import hashlib,json,math,os,pathlib,platform,statistics,subprocess,sys,time
root=pathlib.Path(__file__).resolve().parent
os.chdir(root)
out=root/'raw'
if out.exists() and any(out.iterdir()):
    sys.exit("raw/ already holds a campaign; move it aside or run from a fresh source directory")
out.mkdir(exist_ok=True)
def command(args,log=None):
    try:
        result=subprocess.run(args,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=60)
    except subprocess.TimeoutExpired as e:
        (out/("failure-"+str(time.time_ns())+".txt")).write_text(repr(args)+"\ntimeout after "+str(e.timeout)+"s\n"+(e.stdout or b'').decode(errors='replace'))
        raise RuntimeError("retained timed-out command: "+repr(args)) from e
    if log:
        with log.open('a') as f:
            f.write(result.stdout)
    if result.returncode:
        (out/("failure-"+str(time.time_ns())+".txt")).write_text(repr(args)+"\n"+result.stdout)
        raise RuntimeError("retained failed command: "+repr(args))
    return result.stdout
meta={'hostname':platform.node(),'uname':list(platform.uname()),'rustc':command(['rustc','-Vv']),'cpu':command(['lscpu']),'available_cpus':len(os.sched_getaffinity(0)),'affinity':sorted(os.sched_getaffinity(0)),'flags':'--edition 2024 -O -C target-cpu=native','started_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'files':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [pathlib.Path('src/lib.rs'),pathlib.Path('tests/contract.rs'),pathlib.Path('benches/portfolio.rs'),pathlib.Path('run.py')]}}
(out/'identity.json').write_text(json.dumps(meta,indent=2))
rlib=str(out/'libalgorithms_fst.rlib')
portfolio=out/'portfolio'
command(['rustc','--edition','2024','-O','-C','target-cpu=native','--crate-name','algorithms_fst','--crate-type','rlib','src/lib.rs','-o',rlib])
command(['rustc','--edition','2024','--test','src/lib.rs','-o',str(out/'tests-bin')])
command(['rustc','--edition','2024','--test','tests/contract.rs','--extern','algorithms_fst='+rlib,'-o',str(out/'contract-bin')])
failed=[]
for binary in ['tests-bin','contract-bin']:
    try:
        command([str(out/binary)],log=out/'tests.txt')
    except RuntimeError as e:
        failed.append(e)
if failed:
    raise failed[0]
command(['rustc','--edition','2024','-O','-C','target-cpu=native','benches/portfolio.rs','--extern','algorithms_fst='+rlib,'-o',str(portfolio)])
meta['binary_sha256']=hashlib.sha256(portfolio.read_bytes()).hexdigest()
(out/'identity.json').write_text(json.dumps(meta,indent=2))
candidates=['flat','trie','final','pushed'];cases=['tiny','shared','entropy','random-values','miss','prefix','broad','build'];runs=[]
for block in range(8):
    order=candidates[block%4:]+candidates[:block%4]
    if block>=4: order=order[::-1]
    for case in (cases if block%2==0 else cases[::-1]):
        for pos,name in enumerate(order):
            raw=command([str(portfolio),name,case])
            (out/f'{block}-{case}-{pos}-{name}.json').write_text(raw)
            item=json.loads(raw);item.update(block=block,position=pos,label=name);runs.append(item)
        for label in (['A','B'] if block%2==0 else ['B','A']):
            raw=command([str(portfolio),'flat',case])
            (out/f'{block}-{case}-control-{label}.json').write_text(raw)
            item=json.loads(raw);item.update(block=block,label=label);runs.append(item)
    print('completed block',block,flush=True)
summary={}
for case in cases:
    cells={}
    for name in candidates:
        samples=[r['ns_per_unit'] for r in runs if r['case']==case and r['label']==name]
        cells[name]={'median':statistics.median(samples),'min':min(samples),'max':max(samples),'samples':samples}
    best=min(candidates,key=lambda name:cells[name]['median'])
    rivals=[];ratios={}
    for other in candidates:
        if other==best:continue
        pair=[next(r['ns_per_unit'] for r in runs if r['case']==case and r['label']==other and r['block']==b)/next(r['ns_per_unit'] for r in runs if r['case']==case and r['label']==best and r['block']==b) for b in range(8)]
        ratios[other]=pair
        if min(pair)<=1.03:rivals.append(other)
    aa=[next(r['ns_per_unit'] for r in runs if r['case']==case and r['label']=='A' and r['block']==b)/next(r['ns_per_unit'] for r in runs if r['case']==case and r['label']=='B' and r['block']==b) for b in range(8)]
    checksums={r['checksum'] for r in runs if r['case']==case};assert len(checksums)==1
    summary[case]={'candidates':cells,'lowest_median':best,'selection':best if not rivals else 'unresolved: '+best+' / '+','.join(rivals),'paired_ratios':ratios,'AA_ratios':aa,'checksum':checksums.pop(),'structure':{n:{k:next(r[k] for r in runs if r['case']==case and r['label']==n) for k in ['states','arcs','heap_capacity_bytes']}for n in candidates}}
assert hashlib.sha256(portfolio.read_bytes()).hexdigest()==meta['binary_sha256']
for path,digest in meta['files'].items():assert hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()==digest
(out/'summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps({k:v['selection']for k,v in summary.items()}))
