#!/usr/bin/env python3
"""Fixed exploratory schedule and independent free-interval oracle."""
import hashlib,itertools,json,subprocess,sys
from pathlib import Path

KINDS=['scan','buddy','tree']
def oracle(text,n):
    # Free intervals are split and unioned independently of candidate metadata.
    free=[(0,n)]; live={}; answers=[]
    for line in text.splitlines():
        if line.startswith('STATE '):
            expected=[0]*n
            for a,s in live.values(): expected[a:a+s]=[1]*s
            assert json.loads(line[6:])==expected
        elif line.startswith(('A ','F ')):
            typ,ids,rs,actual=line.split(); i=int(ids);r=int(rs)
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
    result=json.loads(text.splitlines()[-1]);assert result['digest']==digest
    return result

def main():
    binary,out=sys.argv[1:3];target=Path(out);target.parent.mkdir(parents=True,exist_ok=True)
    raw=target.parent/'checks';raw.mkdir(exist_ok=True)
    expected={}
    with target.open('w') as f:
        def run(kind,n,shape,phase,block,check=False):
            cmd=[binary,kind,str(n),shape]+(['check'] if check else [])
            text=subprocess.check_output(cmd,text=True)
            if check:
                (raw/f'{kind}-{n}-{shape}.txt').write_text(text)
                r=oracle(text,n); expected[kind,n,shape]=r['digest']
            else:
                r=json.loads(text);assert r['digest']==expected[kind,n,shape]
            r.update(phase=phase,block=block);f.write(json.dumps(r)+'\n');f.flush()
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
    print('binary_sha256',hashlib.sha256(Path(binary).read_bytes()).hexdigest())
if __name__=='__main__': main()
