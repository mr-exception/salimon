import json,math,struct,pathlib,itertools
import matplotlib.pyplot as plt
from matplotlib.patches import Polygon
root=pathlib.Path(__file__).resolve().parents[2]
names=['water-ice-fragment-shard','water-ice-fragment-cluster','silicate-fragment-slab','silicate-fragment-ridge','iron-fragment','iron-fragment-shard']
def convex(points):
 ps=sorted(set(points))
 def cross(o,a,b):return (a[0]-o[0])*(b[1]-o[1])-(a[1]-o[1])*(b[0]-o[0])
 def chain(seq):
  out=[]
  for p in seq:
   while len(out)>1 and cross(out[-2],out[-1],p)<=0:out.pop()
   out.append(p)
  return out
 return chain(ps)[:-1]+chain(reversed(ps))[:-1]
fig,axes=plt.subplots(2,3,figsize=(12,7))
for ax,name in zip(axes.flat,names):
 b=(root/'client/assets/resources'/name/'model.glb').read_bytes();n=struct.unpack_from('<I',b,12)[0];doc=json.loads(b[20:20+n]);data=b[28+n:]
 points=[]
 for node in doc['nodes']:
  if 'mesh' not in node:continue
  for prim in doc['meshes'][node['mesh']]['primitives']:
   acc=doc['accessors'][prim['attributes']['POSITION']];view=doc['bufferViews'][acc['bufferView']];off=view.get('byteOffset',0)+acc.get('byteOffset',0);stride=view.get('byteStride',12)
   points.extend(struct.unpack_from('<3f',data,off+i*stride) for i in range(acc['count']))
 lo=[min(v[a] for v in points) for a in range(3)];hi=[max(v[a] for v in points) for a in range(3)];box=list(itertools.product(*zip(lo,hi)))
 dot=lambda a,b:sum(x*y for x,y in zip(a,b))
 best=None
 for i in range(1024):
  y=1-2*(i+.5)/1024;r=math.sqrt(1-y*y);angle=i*2.399963229728653;d=(r*math.cos(angle),y,r*math.sin(angle));gap=max(dot(p,d) for p in box)-max(dot(p,d) for p in points)
  if best is None or gap>best[0]:best=gap,d
 gap,d=best;t=(d[2],0,-d[0]);length=math.sqrt(dot(t,t));t=tuple(x/length for x in t)
 project=lambda p:(dot(p,t)*40,dot(p,d)*40)
 ax.add_patch(Polygon(convex(map(project,box)),facecolor='#faebe7',edgecolor='#cf523f',linewidth=1.5,label='Box control'))
 ax.add_patch(Polygon(convex(map(project,points)),facecolor='#9db9d6',edgecolor='#214b72',linewidth=1.5,label='Current convex envelope'))
 old=max(dot(p,d)*40 for p in points);new=max(dot(p,d)*40 for p in box)
 ax.axhline(old,color='#214b72',ls=':',lw=1);ax.axhline(new,color='#cf523f',ls=':',lw=1)
 ax.set_title(name.replace('-fragment','').replace('water-ice','ice')+f'\nSampled support gap: {gap*40:.1f} cm',fontsize=10)
 ax.set_aspect('equal');ax.autoscale();ax.set_xlabel('Tangent projection (cm)');ax.set_ylabel('Support projection (cm)');ax.grid(alpha=.18)
axes.flat[0].legend(fontsize=8,loc='lower left')
fig.suptitle('Why bounding boxes are rejected as fragment contact proxies',fontsize=16)
fig.text(.5,.012,'Geometric projections at a 40 cm control side, viewed along each sampled worst support direction. This is not a native screenshot.',ha='center',fontsize=10)
fig.tight_layout(rect=(0,.04,1,.94));fig.savefig(root/'reports/issue-160/support-comparison.png',dpi=160)
