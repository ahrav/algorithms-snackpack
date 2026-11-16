#!/usr/bin/env python3
"""Fixed exploratory schedule and independent free-interval oracle."""
import hashlib,itertools,json,os,subprocess,sys
from pathlib import Path

if not __debug__:
    raise SystemExit('measure.py needs assertions enabled: run it without -O and with PYTHONOPTIMIZE unset')

KINDS=['scan','buddy','tree']
TIMEOUT_S=float(os.environ.get('TOPIC_051_PROCESS_TIMEOUT_S','60'))
def image(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def trace(n,shape):
    # Declared operations of one BENCHMARK.md workload, in order: (type, ID, request).
    if shape=='holes':
        evens=range(0,n,2)
        return [('A',i,1) for i in range(n)]+[('F',i,0) for i in evens]+[('A',i,2) for i in evens]+[('F',i,0) for i in range(n)]
    ops=[];count=8 if shape=='churn' else n//2
    for rnd in range(8):
        for i in range(count):
            if shape=='tiny': r=1
            elif shape=='mixed': r=(1,3,5,9,17,33)[(i+rnd)%6]
            elif shape=='churn': r=(n//2-1,3,1,n//4+1)[i%4]
            else: raise ValueError(shape)
            ops.append(('A',i,r))
        ops+=[('F',i,0) for i in range(count-1,-1,-1)]
    return ops
def oracle(text,n,shape):
    # Free intervals are split and unioned independently of candidate metadata.
    free=[(0,n)]; live={}; answers=[]
    want=trace(n,shape);lines=text.splitlines()
    # A check output is the declared operations in order, one STATE line and the summary.
    assert len(lines)==len(want)+2,f'{len(lines)} lines for {len(want)} declared operations'
    for index,line in enumerate(lines[:-1]):
        if index==len(want):
            assert line.startswith('STATE '),line[:40]
            expected=[0]*n
            for a,s in live.values(): expected[a:a+s]=[1]*s
            assert json.loads(line[6:])==expected
        else:
            typ,ids,rs,actual=line.split(); i=int(ids);r=int(rs)
            assert (typ,i,r)==want[index],(line,want[index])
            if typ=='A':
                size=1
                while size<r: size*=2
                got=-1
                if r>0 and i not in live:
                    for k,(lo,hi) in enumerate(free):
                        a=((lo+size-1)//size)*size
                        if a+size<=hi:
                            got=a;live[i]=(a,size)
                            free[k:k+1]=([(lo,a)] if lo<a else [])+([(a+size,hi)] if a+size<hi else [])
                            break
            else:
                got=int(i in live)
                if got:
                    a,s=live.pop(i);parts=sorted(free+[(a,a+s)]);free=[]
                    for lo,hi in parts:
                        if free and free[-1][1]==lo: free[-1]=(free[-1][0],hi)
                        else: free.append((lo,hi))
            assert int(actual)==got,(line,got)
            answers.append(got)
    digest=0
    for x in answers: digest=(digest*31+abs(x))%(1<<64)
    result=json.loads(lines[-1]);assert result['digest']==digest and result['ops']==len(want)
    return result

def main():
    binary,out=sys.argv[1:3];target=Path(out)
    raw=target.parent/'checks'
    for old in (target,raw):
        if old.exists(): raise SystemExit(f'{old} exists: archive it and remove it before another run')
    pinned=image(binary)
    raw.mkdir(parents=True)
    expected={}
    with target.open('x') as f:
        def run(kind,n,shape,phase,block,check=False):
            # Every attempt leaves one classified line; the first one that is not ok halts the run.
            cmd=[binary,kind,str(n),shape]+(['check'] if check else [])
            # One image runs the whole campaign: a changed or unreadable binary stops it before the launch.
            try: current=image(binary)
            except OSError as error: current=repr(error)
            if current!=pinned: raise SystemExit(f'{binary} is {current} before {phase} {kind} {n} {shape}, pinned sha256 {pinned}; run stopped')
            try:
                p=subprocess.run(cmd,capture_output=True,timeout=TIMEOUT_S,check=False)
                stdout,stderr=p.stdout,p.stderr
                r=dict(status='ok' if p.returncode==0 else 'failed',exit=p.returncode)
            except subprocess.TimeoutExpired as expired:
                stdout,stderr=expired.stdout or b'',expired.stderr or b''
                r=dict(status='timeout',timeout_s=TIMEOUT_S)
            text=stdout.decode(errors='replace')
            if check: (raw/f'{kind}-{n}-{shape}.txt').write_text(text)
            if r['status']=='ok':
                try:
                    data=oracle(text,n,shape) if check else json.loads(text)
                    assert [data[k] for k in ('kind','n','shape')]==[kind,n,shape],'kind, n or shape differs from the request'
                    agreed=[data[k] for k in ('digest','ops','work')]
                    # A timed process repeats the digest, operation count and work of its verified check process.
                    assert agreed==expected.setdefault((kind,n,shape),agreed),'digest, ops or work differs from the check process'
                    assert type(data['ns']) is int and data['ns']>0,'ns is not a positive integer'
                    assert type(data['setup_ns']) is int and data['setup_ns']>=0,'setup_ns is not a nonnegative integer'
                    r=dict(data,status='ok')
                except Exception as error:  # any parse, oracle or agreement failure is a rejected attempt
                    r.update(status='rejected',error=repr(error))
            if r['status']!='ok':
                r=dict(kind=kind,n=n,shape=shape,**r,command=cmd,stdout=text,stderr=stderr.decode(errors='replace'))
            r.update(phase=phase,block=block);f.write(json.dumps(r)+'\n');f.flush()
            if r['status']!='ok': raise SystemExit(f"{phase} {kind} {n} {shape} {r['status']}; retained in {target}")
        for n in [64,4096]:
            for shape in ['tiny','mixed','holes','churn']:
                for k in KINDS: run(k,n,shape,'check',-1,True)
                for k in KINDS: run(k,n,shape,'warmup',-1)
                for block,order in enumerate(itertools.permutations(KINDS)):
                    for k in order: run(k,n,shape,'measure',block)
        for block in range(6):
            for label in (['A','B'] if block%2==0 else ['B','A']):
                run('scan',4096,'mixed','AA'+label,block)
    print('PASS 24 complete independent-oracle traces; 204 process records')
    print('binary_sha256',pinned)
if __name__=='__main__': main()
