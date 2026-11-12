#!/usr/bin/env python3
"""Fixed balanced process campaign with independent correctness checks."""
import bisect, collections, hashlib, json, math, os, pathlib, platform, statistics, subprocess, sys, time
if not __debug__:
    raise SystemExit('run.py needs assertions enabled: run python3 run.py without -O and with PYTHONOPTIMIZE unset')
ROOT=pathlib.Path(__file__).resolve().parent
OUT=ROOT/'raw'
TIMEOUT=float(os.environ.get('TOPIC_046_PROCESS_TIMEOUT_S',120))
MASK=(1<<64)-1
CANDIDATES=['exact-frequency','count-min','exact-distinct','hll','exact-quantile','rank-summary']
WORKLOADS=[(128,'uniform'),(4096,'uniform'),(16384,'uniform'),(16384,'skew'),(16384,'sorted'),(16384,'reverse')]
def mix(x,seed):
    x=(x+seed+0x9e3779b97f4a7c15)&MASK
    x=((x^(x>>30))*0xbf58476d1ce4e5b9)&MASK
    x=((x^(x>>27))*0x94d049bb133111eb)&MASK
    return x^(x>>31)
def inputs(n,shape):
    return [mix(i,123)%4096 if shape=='uniform' else (0 if i%10<9 else mix(i,123)%4096) if shape=='skew' else i if shape=='sorted' else n-1-i for i in range(n)]
def check(r):
    data=inputs(r['n'],r['shape']); n=len(data); c=r['candidate']; seed=r['seed']
    counts=collections.Counter(data); ordered=sorted(data)
    assert r['bytes']=={'exact-frequency':16*len(counts),'count-min':10240,'exact-distinct':8*len(counts),'hll':1024,'exact-quantile':8*n,'rank-summary':8*len(r['state'])}[c]
    if c in ('exact-frequency','count-min'):
        exact=[counts[i] for i in range(4096)]
        if c=='exact-frequency': assert r['values']==exact
        else:
            table=[0]*1280
            for x,count in counts.items():
                for row in range(5): table[row*256+(mix(x,seed+row)&255)]+=count
            assert r['state']==table
            expected=[min(table[row*256+(mix(x,seed+row)&255)] for row in range(5)) for x in range(4096)]
            assert r['values']==expected and all(a>=b for a,b in zip(expected,exact))
    elif c=='exact-distinct': assert r['estimate']==len(counts)
    elif c=='hll':
        registers=[0]*1024
        for x in counts:
            h=mix(x,seed); j=h>>54; rest=h&((1<<54)-1)
            registers[j]=max(registers[j],55-rest.bit_length())
        assert r['state']==registers
        z=registers.count(0); raw=(.7213/(1+1.079/1024))*1024**2/sum(2**-v for v in registers)
        estimate=1024*math.log(1024/z) if raw<=2560 and z else raw
        assert math.isclose(r['estimate'],estimate,rel_tol=1e-12,abs_tol=1e-10)
    else:
        assert len(r['values'])==101
        for q,x in enumerate(r['values']):
            target=1+(n-1)*q//100
            lo=bisect.bisect_left(ordered,x)+1; hi=bisect.bisect_right(ordered,x)
            allowance=0 if c=='exact-quantile' else n//50
            assert lo<=hi and lo<=target+allowance and hi+allowance>=target,(r['shape'],q,x,lo,hi,target)
        assert r['values'][0]==ordered[0] and r['values'][-1]==ordered[-1]
        if c=='rank-summary':
            state=r['state']; assert len(state)%3==0
            cumulative=0; previous=-1
            for i in range(0,len(state),3):
                value,gap,delta=state[i:i+3]; cumulative+=gap
                assert value in counts and value>=previous and gap>=1
                assert gap+delta<=max(1,n//25)
                assert cumulative<=bisect.bisect_right(ordered,value)
                assert cumulative+delta>=bisect.bisect_left(ordered,value)+1
                previous=value
            assert cumulative==n
    return True
def cmd(*args): return subprocess.check_output(args,text=True,stderr=subprocess.STDOUT,cwd=ROOT)
def one(c,n,shape,seed,repeats,label):
    args=[str(ROOT/'sketch-lab'),c,str(n),shape,str(seed),str(repeats)]
    if hasattr(os,'sched_getaffinity'): args=['taskset','-c',str(min(os.sched_getaffinity(0)))]+args
    path=OUT/f'{label}.json'
    def text(captured):
        return (captured or b'').decode(errors='replace')
    def halt(status,**detail):
        path.write_text(json.dumps({'label':label,'status':status,'command':args,**detail}))
        return SystemExit(f'attempt {status}; retained in {path}')
    try:
        p=subprocess.run(args,capture_output=True,timeout=TIMEOUT)
    except subprocess.TimeoutExpired as expired:
        raise halt('timeout',timeout_s=TIMEOUT,stdout=text(expired.stdout),stderr=text(expired.stderr)) from None
    stdout,stderr=text(p.stdout),text(p.stderr)
    if p.returncode:
        raise halt('failed',exit=p.returncode,stdout=stdout,stderr=stderr)
    try:
        r=json.loads(stdout)
        assert (r['candidate'],r['n'],r['shape'],r['seed'],r['repeats'])==(c,n,shape,seed,repeats)
        assert type(r['ns']) in (int,float) and math.isfinite(r['ns']) and r['ns']>0
        check(r)
    except Exception as error:
        raise halt('rejected',error=repr(error),stdout=stdout,stderr=stderr) from None
    r.update(label=label,status='ok',oracle='PASS',command=args)
    path.write_text(json.dumps(r))
    return r

def main():
    earlier=[str(p) for p in (OUT,ROOT/'results.json',ROOT/'manifest.json') if p.exists()]
    if earlier:
        raise SystemExit('outputs of an earlier campaign exist; archive them outside the repository, then remove: '+' '.join(earlier))
    OUT.mkdir()
    flags=['--edition','2024','-C','opt-level=3','-C','target-cpu=native']
    (OUT/'build.txt').write_text(cmd('rustc',*flags,'main.rs','-o','sketch-lab'))
    (OUT/'contracts.txt').write_text(cmd('rustc','--edition','2024','--test','-O','src/lib.rs','-o','contracts')+cmd('./contracts'))
    (OUT/'generated.s').write_text(cmd('rustc',*flags,'--emit=asm','main.rs','-o','generated.s')+(ROOT/'generated.s').read_text())
    env={'hostname':platform.node(),'uname':list(platform.uname()),'cpus_available':sorted(os.sched_getaffinity(0)) if hasattr(os,'sched_getaffinity') else [],'cpuinfo':pathlib.Path('/proc/cpuinfo').read_text(),'rustc':cmd('rustc','-vV'),'target_features':cmd('rustc','-C','target-cpu=native','--print','cfg'),'flags':flags,'python':sys.version,'start':time.time(),'hashes':{f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest() for f in ['src/lib.rs','main.rs','run.py','sketch-lab']}}
    (OUT/'environment.json').write_text(json.dumps(env,indent=2))
    records=[]; accuracy=[]
    for wi,(n,shape) in enumerate(WORKLOADS):
        for c in CANDIDATES: one(c,n,shape,7,1,f'warmup-{wi}-{c}')
        for block in range(6):
            # Pair members adjacent; rotate contract order; reverse pair order in odd blocks.
            pairs=[CANDIDATES[i:i+2] for i in range(0,6,2)]
            pairs=pairs[block%3:]+pairs[:block%3]
            for pair in pairs:
                for c in (pair if block%2==0 else pair[::-1]):
                    r=one(c,n,shape,7,32 if n==128 else 4,f'main-{wi}-{block}-{c}'); r['block']=block; records.append(r)
        for seed in range(16):
            r=one('hll',n,shape,seed,1,f'accuracy-{wi}-{seed}')
            accuracy.append({'n':n,'shape':shape,'seed':seed,'distinct':len(set(inputs(n,shape))),'estimate':r['estimate']})
    aa=[]
    for b in range(6):
        for arm in (['A','B'] if b%2==0 else ['B','A']):
            r=one('exact-frequency',16384,'uniform',7,4,f'aa-{b}-{arm}'); r['block']=b;r['arm']=arm;aa.append(r)
    result={'cells':[],'accuracy':accuracy,'aa':aa,'processes':len(list(OUT.glob('*.json')))-1,'main_processes':len(records),'stopping':'6 paired processes per cell, every contrast >5%, descriptive only'}
    for n,shape in WORKLOADS:
        for i in range(0,6,2):
            a,b=CANDIDATES[i:i+2]; ra=[r for r in records if r['n']==n and r['shape']==shape and r['candidate']==a]; rb=[r for r in records if r['n']==n and r['shape']==shape and r['candidate']==b]
            ratio=[y['ns']/x['ns'] for x,y in zip(ra,rb)]
            winner=a if all(v>1.05 for v in ratio) else b if all(v<1/1.05 for v in ratio) else 'unresolved'
            result['cells'].append({'n':n,'shape':shape,'pair':[a,b],'winner':winner,'ratio_b_over_a':ratio,'statistics':{c:{'median_ns':statistics.median(r['ns'] for r in rr),'min_ns':min(r['ns'] for r in rr),'max_ns':max(r['ns'] for r in rr),'retained_bytes':rr[0]['bytes']} for c,rr in [(a,ra),(b,rb)]}})
    result['aa_ratios']=[next(r['ns'] for r in aa if r['block']==b and r['arm']=='B')/next(r['ns'] for r in aa if r['block']==b and r['arm']=='A') for b in range(6)]
    (ROOT/'results.json').write_text(json.dumps(result,indent=2))
    manifest={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(OUT.iterdir()) if p.is_file()}
    (ROOT/'manifest.json').write_text(json.dumps(manifest,indent=2))
    print(json.dumps({'processes':result['processes'],'main':len(records),'winners':dict(collections.Counter(c['winner'] for c in result['cells'])),'aa':result['aa_ratios']}))
if __name__=='__main__': main()
