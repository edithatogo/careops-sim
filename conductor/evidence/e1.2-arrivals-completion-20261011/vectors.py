import json
MASK=(1<<64)-1
def mix(x):
 x=(x+0x9E3779B97F4A7C15)&MASK
 x=((x^(x>>30))*0xBF58476D1CE4E5B9)&MASK
 x=((x^(x>>27))*0x94D049BB133111EB)&MASK
 return x^(x>>31)
def derive(run):
 x=(run+0xA8E51B2C4D6F9013)&MASK
 x^=mix((0x434152454F505341+0x9E3779B97F4A7C15)&MASK)
 x=((x<<17)|(x>>47))&MASK
 x=(x+mix(0xBF58476D1CE4E5B9)*0x94D049BB133111EB)&MASK
 return mix(x^0xD6E8FEB86659FD93)
vectors=[]
for root in [11,23]:
 state=derive(root);draws=[]
 for i in range(5):
  state=mix(state);draws.append(state)
 vectors.append(dict(seed=root,derived=derive(root),raw_draws=draws,width10_threshold=6,offsets=[v%10 for v in draws]))
print(json.dumps(dict(scope="Independent pinned-RNG arithmetic, not native generator execution",vectors=vectors),indent=2))
