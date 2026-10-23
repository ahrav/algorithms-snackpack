"""Fixed process-replicated dictionary campaign. Python standard library only."""
import hashlib,json,os,pathlib,platform,statistics,subprocess,sys,time
root=pathlib.Path(__file__).resolve().parents[1]
out=pathlib.Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
for stale in ['metadata.json','libtopic024.rlib','dictionary','unit-tests','contract-tests','tests.txt','schedule.json','raw.jsonl','attempts.jsonl','failure.json','summary.json']:
    (out/stale).unlink(missing_ok=True)
os.chdir(root)
TIMEOUT=600
def call(args): return subprocess.check_output(args,text=True,stderr=subprocess.STDOUT)
files=['src/lib.rs','tests/contracts.rs','benches/dictionary.rs','scripts/run.py']
meta={"hostname":platform.node(),"architecture":platform.machine(),"uname":list(platform.uname()),"cpu":call(['lscpu']),"available_cpus":len(os.sched_getaffinity(0)),"affinity":sorted(os.sched_getaffinity(0)),"rustc":call(['rustc','-Vv']),"features":call(['rustc','--print','cfg','-C','target-cpu=native']),"flags":"--edition=2024 -C opt-level=3 -C target-cpu=native; no LTO","timeout_s":TIMEOUT,"files":{p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest() for p in files}}
(out/'metadata.json').write_text(json.dumps(meta,indent=2))
lib=f'topic024_tries_radix_trees_and_art={out/"libtopic024.rlib"}'
call(['rustc','--edition=2024','--crate-type=rlib','--crate-name','topic024_tries_radix_trees_and_art','-C','opt-level=3','-C','target-cpu=native','src/lib.rs','-o',out/'libtopic024.rlib'])
call(['rustc','--edition=2024','-C','opt-level=3','-C','target-cpu=native','benches/dictionary.rs','--extern',lib,'-o',out/'dictionary'])
call(['rustc','--edition=2024','--test','src/lib.rs','-o',out/'unit-tests'])
call(['rustc','--edition=2024','--test','tests/contracts.rs','--extern',lib,'-o',out/'contract-tests'])
tests=[subprocess.run([out/name],text=True,capture_output=True) for name in ['unit-tests','contract-tests']]
(out/'tests.txt').write_text(''.join(t.stdout+t.stderr for t in tests))
for t in tests: t.check_returncode()
meta['binary_sha256']=hashlib.sha256((out/'dictionary').read_bytes()).hexdigest();(out/'metadata.json').write_text(json.dumps(meta,indent=2))
cases=['tiny','shared','wide','miss','prefix256','prefix_all','build_shared','build_wide']
labels=['flat','tree','trie','radix','treeA','treeB']; schedule=[]; records=[]
for block in range(6):
    shift=block//2
    cells=cases[shift:]+cases[:shift]
    for case in cells:
        order=labels[shift:]+labels[:shift]
        if block%2:order=list(reversed(order))
        for label in order:
            schedule.append({'block':block,'case':case,'label':label,'candidate':'tree' if label.startswith('tree') else label})
(out/'schedule.json').write_text(json.dumps(schedule,indent=2))
with (out/'raw.jsonl').open('w') as f,(out/'attempts.jsonl').open('w') as attempts:
    for s in schedule:
        t=time.time()
        try: p=subprocess.run([out/'dictionary',s['candidate'],s['case']],capture_output=True,text=True,timeout=TIMEOUT)
        except subprocess.TimeoutExpired as e:
            attempts.write(json.dumps({**s,'stdout':str(e.stdout),'stderr':str(e.stderr),'returncode':'timeout','wall_seconds':time.time()-t})+'\n');raise SystemExit(1) from e
        attempt={**s,'stdout':p.stdout,'stderr':p.stderr,'returncode':p.returncode,'wall_seconds':time.time()-t}
        attempts.write(json.dumps(attempt)+'\n');attempts.flush()
        if p.returncode:
            (out/'failure.json').write_text(json.dumps(attempt));raise SystemExit(1)
        row={**s,**json.loads(p.stdout),'started':t};records.append(row);f.write(json.dumps(row)+'\n');f.flush()
summary={}
for case in cases:
    groups={label:[r['ns'] for r in records if r['case']==case and r['label']==label] for label in labels}
    med={k:statistics.median(v) for k,v in groups.items()};best=min(labels[:4],key=lambda k:med[k])
    ratios={k:[a/b for a,b in zip(groups[k],groups[best])] for k in labels[:4] if k!=best}
    resolved=all(min(v)>1.03 for v in ratios.values())
    aa=[a/b for a,b in zip(groups['treeA'],groups['treeB'])]
    summary[case]={'fastest_median':best,'selection':best if resolved else 'unresolved','median_ns':med,'range_ns':{k:[min(v),max(v)] for k,v in groups.items()},'rival_over_best_ranges':{k:[min(v),max(v)] for k,v in ratios.items()},'AA_ratio_range':[min(aa),max(aa)]}
    checks={r['checksum'] for r in records if r['case']==case};assert len(checks)==1
(out/'summary.json').write_text(json.dumps(summary,indent=2))
print(json.dumps({'processes':len(records),'selection':{k:v['selection'] for k,v in summary.items()}}))
