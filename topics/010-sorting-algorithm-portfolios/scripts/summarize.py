import json,math,statistics,sys,itertools
from pathlib import Path
def cf(a,b,x):
 qab=a+b;qap=a+1;qam=a-1;c=1.0;d=1-qab*x/qap;d=1/max(d,1e-300);h=d
 for m in range(1,300):
  m2=2*m;aa=m*(b-m)*x/((qam+m2)*(a+m2));d=1+aa*d;c=1+aa/c
  if abs(d)<1e-300:d=1e-300
  if abs(c)<1e-300:c=1e-300
  d=1/d;h*=d*c;aa=-(a+m)*(qab+m)*x/((a+m2)*(qap+m2));d=1+aa*d;c=1+aa/c
  if abs(d)<1e-300:d=1e-300
  if abs(c)<1e-300:c=1e-300
  d=1/d;delta=d*c;h*=delta
  if abs(delta-1)<1e-14:return h
 raise ValueError('beta no convergence')
def beta(x,a,b):
 if x==0:return 0.0
 if x==1:return 1.0
 bt=math.exp(math.lgamma(a+b)-math.lgamma(a)-math.lgamma(b)+a*math.log(x)+b*math.log1p(-x))
 return bt*cf(a,b,x)/a if x<(a+1)/(a+b+2) else 1-bt*cf(b,a,1-x)/b
def cdf(t,df):return 1-.5*beta(df/(df+t*t),df/2,.5)
def ppf(p,df):
 lo,hi=0.,1000.
 for _ in range(100):
  mid=(lo+hi)/2
  if cdf(mid,df)<p:lo=mid
  else:hi=mid
 return (lo+hi)/2
assert abs(ppf(.975,1)-math.tan(math.pi*.475))<1e-8
assert abs(ppf(.975,11)-2.200985160)<1e-8
base=Path(sys.argv[1]);critical=ppf(1-.05/(2*620),11)
for host in ['arm','xxl']:
 rec=[json.loads(x) for x in (base/(host+'-attempts.jsonl')).read_text().splitlines()];cells={}
 for cell in dict.fromkeys(r['cell'] for r in rec):
  methods={}
  for m in dict.fromkeys(r['method'] for r in rec if r['cell']==cell):
   values=[r['ns_per_call'] for r in rec if r['cell']==cell and r['method']==m];methods[m]={'gm_ns':math.exp(statistics.mean(map(math.log,values))),'median_ns':statistics.median(values),'min_ns':min(values),'max_ns':max(values),'values':values}
  fastest=min(methods,key=lambda m:methods[m]['gm_ns']);unresolved=[];contrasts=[]
  for a,b in itertools.combinations(methods,2):
   logs=[math.log(x/y) for x,y in zip(methods[a]['values'],methods[b]['values'])];mu=statistics.mean(logs);se=statistics.stdev(logs)/math.sqrt(12)
   contrasts.append({'a':a,'b':b,'ratio_a_over_b':math.exp(mu),'simultaneous_95_bounds':[math.exp(mu-critical*se),math.exp(mu+critical*se)]})
  for other in methods:
   if other==fastest:continue
   logs=[math.log(x/y) for x,y in zip(methods[other]['values'],methods[fastest]['values'])];lo=math.exp(statistics.mean(logs)-critical*statistics.stdev(logs)/math.sqrt(12))
   if lo<=1.05:unresolved.append(other)
  for m in methods:del methods[m]['values']
  cells[cell]={'fastest_point_estimate':fastest,'unique_winner':not unresolved,'unresolved_rivals':unresolved,'methods':methods,'contrasts':contrasts}
 
 for cell,c in cells.items():
  eligible=[m for m in c['methods'] if m in ['stable','merge','radix','dispatch','groups']]
  fastest=min(eligible,key=lambda m:c['methods'][m]['gm_ns']); unresolved=[]
  for other in eligible:
   if other==fastest:continue
   contrast=next(x for x in c['contrasts'] if {x['a'],x['b']}=={other,fastest})
   lo,hi=contrast['simultaneous_95_bounds']; lower=lo if contrast['a']==other else 1/hi
   if lower<=1.05:unresolved.append(other)
  c['stable_selection']={'fastest_point_estimate':fastest,'unique_winner':not unresolved,'unresolved_rivals':unresolved}
 summary={'host':host,'processes':len(rec),'critical_t':critical,'family':620,'cells':cells}
 (base/(host+'-summary.json')).write_text(json.dumps(summary,indent=2))
 print(host,'t',round(critical,4))
 for cell,c in cells.items():print(cell,c['fastest_point_estimate'], 'winner' if c['unique_winner'] else 'unresolved:'+','.join(c['unresolved_rivals']), {m:round(x['gm_ns']) for m,x in c['methods'].items()})
