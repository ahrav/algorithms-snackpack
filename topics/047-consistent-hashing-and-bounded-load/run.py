#!/usr/bin/env python3
"""Frozen order-balanced process campaign and independent full-vector oracle."""
import bisect, collections, functools, hashlib, json, os, pathlib, platform, statistics, subprocess, sys, time
ROOT=pathlib.Path(__file__).resolve().parent
OUT=ROOT/'raw'; OUT.mkdir(exist_ok=True)
MASK=(1<<64)-1
CANDIDATES=['modulo','ring1','ring32','hrw','bounded']
WORKLOADS=[(n,k,'uniform') for n in [4,32,128] for k in [32,32768]]+[(32,4096,'cluster')]
def h(x):
    x=(x+0x9e3779b97f4a7c15)&MASK
    x=((x^(x>>30))*0xbf58476d1ce4e5b9)&MASK
    x=((x^(x>>27))*0x94d049bb133111eb)&MASK
    return x^(x>>31)
@functools.cache
def tokens(ids,v): return sorted((h(i^h(t^0xd1b54a32d192ed03)),i,t) for i in ids for t in range(v))
@functools.cache
def inputs(n,k,shape):
    if shape=='uniform':return tuple(h(i) for i in range(k))
    ids=tuple(i*17+7 for i in range(n)); ts=tokens(ids,32); positions=[x[0] for x in ts]
    keys=[];x=0
    while len(keys)<k:
        if ts[bisect.bisect_left(positions,h(x))%len(ts)][1]==7:keys.append(x)
        x+=1
    return tuple(keys)
@functools.cache
def oracle(c,ids,keys):
    ids=tuple(sorted(ids)); counts=collections.Counter(); out=[]; probes=0; cap=(5*len(keys)+4*len(ids)-1)//(4*len(ids))
    if c in ('ring1','ring32','bounded'):
        ts=tokens(ids,1 if c=='ring1' else 32); positions=[t[0] for t in ts]
    for key in keys:
        if c=='modulo':owner=ids[h(key)%len(ids)]
        elif c=='hrw':owner=min(ids,key=lambda i:(-h(key^h(i^0xa0761d6478bd642f)),i))
        else:
            j=bisect.bisect_left(positions,h(key))%len(ts);probes+=1
            if c=='bounded':
                while counts[ts[j][1]]>=cap:j=(j+1)%len(ts);probes+=1
            owner=ts[j][1]
        out.append(owner);counts[owner]+=1
    return tuple(out),probes

def check(r):
    ids=tuple(i*17+7 for i in range(r['n'])); keys=inputs(r['n'],r['k'],r['shape']); c=r['candidate']
    assert tuple(r['keys'])==keys and len(set(keys))==len(keys)
    transitions={'owners':ids,'added':ids+(r['n']*17+7,),'removed':tuple(x for x in ids if x!=24),'replaced':tuple(x for x in ids if x!=24)+(r['n']*17+7,)}
    for field,owners in transitions.items():
        expected,_=oracle(c,owners,keys);assert tuple(r[field])==expected,(c,field)
        if c=='bounded':assert max(collections.Counter(expected).values(),default=0)<=(5*len(keys)+4*len(owners)-1)//(4*len(owners))
    if c in ('ring1','ring32','hrw'):
        assert all(a==b or b==r['n']*17+7 for a,b in zip(r['owners'],r['added']))
        assert all(a==b or a==24 for a,b in zip(r['owners'],r['removed']))
    return True

def cmd(*args):return subprocess.check_output(args,text=True,stderr=subprocess.STDOUT)
def one(c,n,k,shape,phase,reps,label):
    args=[str(ROOT/'placement-lab'),c,str(n),str(k),shape,phase,str(reps)]
    if hasattr(os,'sched_getaffinity'):args=['taskset','-c',str(min(os.sched_getaffinity(0)))]+args
    p=subprocess.run(args,capture_output=True,text=True,timeout=120)
    if p.returncode:
        (OUT/f'{label}-failure.json').write_text(json.dumps({'args':args,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr}));raise RuntimeError(label)
    r=json.loads(p.stdout);check(r);r.update(label=label,command=args,oracle='PASS')
    (OUT/f'{label}.json').write_text(json.dumps(r));return r

def summary(rr):return {'median_ns':statistics.median(r['ns'] for r in rr),'min_ns':min(r['ns'] for r in rr),'max_ns':max(r['ns'] for r in rr),'payload_bytes':rr[0]['bytes']}
def main():
    flags=['--edition','2024','-C','opt-level=3','-C','target-cpu=native']
    (OUT/'build.txt').write_text(cmd('rustc',*flags,'main.rs','-o','placement-lab'))
    (OUT/'unit-tests.txt').write_text(cmd('rustc','--edition','2024','--test','-O','src/lib.rs','-o','unit-tests')+cmd('./unit-tests'))
    (OUT/'contracts.txt').write_text(cmd('rustc','--edition','2024','--crate-type=rlib','--crate-name','topic047_placement','-O','src/lib.rs','-o','libtopic047_placement.rlib')+cmd('rustc','--edition','2024','--test','-O','tests/contracts.rs','--extern','topic047_placement=libtopic047_placement.rlib','-o','contracts')+cmd('./contracts'))
    cmd('rustc',*flags,'--emit=asm','main.rs','-o',str(OUT/'generated.s'))
    env={'hostname':platform.node(),'uname':list(platform.uname()),'available_cpus':sorted(os.sched_getaffinity(0)),'cpuinfo':pathlib.Path('/proc/cpuinfo').read_text(),'rustc':cmd('rustc','-vV'),'target_features':cmd('rustc','-C','target-cpu=native','--print','cfg'),'flags':flags,'python':sys.version,'time':time.time(),'hashes':{f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest() for f in ['src/lib.rs','tests/contracts.rs','main.rs','run.py','placement-lab']}}
    (OUT/'environment.json').write_text(json.dumps(env,indent=2));records=[];cells=[]
    jobs=[(n,k,s,'batch') for n,k,s in WORKLOADS]+[(n,32,'uniform','build') for n in [4,32,128]]
    for wi,(n,k,shape,phase) in enumerate(jobs):
        candidates=CANDIDATES if phase=='batch' else CANDIDATES[:-1]
        for c in candidates:one(c,n,k,shape,phase,1,f'warm-{wi}-{c}')
        for block in range(6):
            for c in (candidates if block%2==0 else candidates[::-1]):
                reps=512 if phase=='build' or k==32 else 4
                r=one(c,n,k,shape,phase,reps,f'main-{wi}-{block}-{c}');r['block']=block;records.append(r)
        groups=[candidates] if phase=='build' else [CANDIDATES[:-1],['ring1','ring32','hrw'],['bounded']]
        for group in groups:
            data={c:sorted([r for r in records if (r['n'],r['k'],r['shape'],r['phase'],r['candidate'])==(n,k,shape,phase,c)],key=lambda r:r['block']) for c in group}
            fastest=min(group,key=lambda c:statistics.median(r['ns'] for r in data[c]))
            winner=fastest if len(group)==1 or all(all(a['ns']*1.05<b['ns'] for a,b in zip(data[fastest],data[c])) for c in group if c!=fastest) else 'unresolved'
            cells.append({'n':n,'k':k,'shape':shape,'phase':phase,'group':group,'winner':winner,'stats':{c:summary(data[c]) for c in group}})
    aa=[]
    for block in range(6):
        for arm in (['A','B'] if block%2==0 else ['B','A']):
            r=one('ring32',32,32768,'uniform','batch',4,f'aa-{block}-{arm}');r.update(block=block,arm=arm);aa.append(r)
    ledgers=[]
    for n,k,shape in WORKLOADS:
        keys=inputs(n,k,shape);ids=tuple(i*17+7 for i in range(n))
        for c in CANDIDATES:
            r=next(r for r in records if (r['n'],r['k'],r['shape'],r['phase'],r['candidate'])==(n,k,shape,'batch',c))
            counts=collections.Counter(r['owners']); weights=collections.Counter()
            # Key zero has 9K work units, each remaining key has one.
            for i,owner in enumerate(r['owners']):weights[owner]+=9*k if i==0 else 1
            ledger={'n':n,'k':k,'shape':shape,'candidate':c,'max_count':max(counts.values()),'count_ratio':max(counts.values())*n/k,'max_work_fraction':max(weights.values())/(10*k-1),'reference_token_visits':oracle(c,ids,keys)[1]}
            for field in ['added','removed','replaced']:
                changed=[i for i,(a,b) in enumerate(zip(r['owners'],r[field])) if a!=b]
                ledger[field]={'keys':len(changed),'key_fraction':len(changed)/k,'weighted_fraction':sum(9*k if i==0 else 1 for i in changed)/(10*k-1)}
            ledgers.append(ledger)
    result={'cells':cells,'ledgers':ledgers,'aa_ratios':[next(r['ns'] for r in aa if r['block']==b and r['arm']=='B')/next(r['ns'] for r in aa if r['block']==b and r['arm']=='A') for b in range(6)],'main_processes':len(records),'warmups':sum(len(CANDIDATES) if j[3]=='batch' else 4 for j in jobs),'aa_processes':len(aa),'criterion':'six balanced process blocks, all paired differences >5%, descriptive screen'}
    (ROOT/'results.json').write_text(json.dumps(result,indent=2))
    (ROOT/'manifest.json').write_text(json.dumps({str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(OUT.iterdir()) if p.is_file()},indent=2))
    print(json.dumps({'processes':len(records)+result['warmups']+len(aa),'winners':dict(collections.Counter(c['winner'] for c in cells)),'aa':result['aa_ratios']}))
if __name__=='__main__':main()
