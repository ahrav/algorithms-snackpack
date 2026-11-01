import hashlib
H=lambda b:hashlib.sha256(b).digest()
def root(xs):
 if len(xs)==1:return H(b'\x00'+xs[0])
 m=len(xs)//2
 return H(b'\x01'+root(xs[:m])+root(xs[m:]))
x=[b'A',b'B',b'C',b'D']
a,b,c,d=[H(b'\x00'+v) for v in x]
left=H(b'\x01'+a+b)
c2=H(b'\x00'+b'C2')
r2=H(b'\x01'+left+H(b'\x01'+c2+d))
assert r2==root([b'A',b'B',b'C2',b'D'])
assert r2!=root(x)
print('PASS independent recursive root and cached-path update; rebuild=7 path=3 hashes; root='+r2.hex())
