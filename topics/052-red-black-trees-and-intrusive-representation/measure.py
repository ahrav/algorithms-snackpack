#!/usr/bin/env python3
"""Fixed paired schedule; independent set/sort oracle checks every observable result."""
import hashlib,json,os,subprocess,sys
from pathlib import Path
KINDS=['flat','boxed','arena','btree']
def oracle(text):
    lines=text.splitlines();fields={s.split(' ',1)[0]:s.split(' ',1)[1] for s in lines[:-1]}
    keys=json.loads(fields['KEYS']);queries=json.loads(fields['QUERIES'])
    present=set();births=[];answers=[]
    for key in keys:
        yes=key not in present;answers.append(yes)
        if yes: births.append(key)
        present.add(key)
    assert json.loads(fields['INSERTS'])==answers
    assert json.loads(fields['SORTED'])==sorted(present)
    r=json.loads(lines[-1]);assert r['added']==len(present)
    assert json.loads(fields['SLOTS'])==(births if r['kind']=='arena' else [])
    hits=''.join('1' if key in present else '0' for key in queries);assert hits==fields['HITS']
    digest=0
    for hit in hits: digest=(digest*31+int(hit))%2**64
    assert digest==r['digest'];return r

def main():
    binary,out=sys.argv[1:3];target=Path(out);target.parent.mkdir(parents=True,exist_ok=True)
    checks=target.parent/'checks';checks.mkdir(exist_ok=True)
    prefix=['taskset','-c',str(min(os.sched_getaffinity(0)))] if hasattr(os,'sched_getaffinity') else []
    expected={}
    with target.open('w') as f:
        def run(kind,n,shape,access,phase,block):
            cmd=prefix+[binary,kind,str(n),shape,access]+(['check'] if phase=='check' else [])
            text=subprocess.check_output(cmd,text=True)
            if phase=='check':
                (checks/f'{kind}-{n}-{shape}-{access}.txt').write_text(text)
                r=oracle(text);expected[kind,n,shape,access]=(r['added'],r['digest'])
            else:
                r=json.loads(text);assert (r['added'],r['digest'])==expected[kind,n,shape,access]
            r.update(phase=phase,block=block);f.write(json.dumps(r)+'\n');f.flush()
        for n in [32,4096]:
            for shape in ['ascending','descending','duplicates']:
                for access in ['dispersed','hot']:
                    for k in KINDS:run(k,n,shape,access,'check',-1)
                    for k in KINDS:run(k,n,shape,access,'warmup',-1)
                    for block in range(6):
                        shift=block//2;order=KINDS[shift:]+KINDS[:shift]
                        if block%2:order=order[::-1]
                        for k in order:run(k,n,shape,access,'measure',block)
        for block in range(6):
            for phase in (['AAA','AAB'] if block%2==0 else ['AAB','AAA']):run('arena',4096,'descending','dispersed',phase,block)
    print('PASS 48 full independent-oracle traces; 396 fresh processes')
    print('binary_sha256',hashlib.sha256(Path(binary).read_bytes()).hexdigest())
if __name__=='__main__':main()
