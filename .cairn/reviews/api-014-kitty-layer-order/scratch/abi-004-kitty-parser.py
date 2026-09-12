import sys, json, hashlib, random
from pathlib import Path
sys.path.append('/tmp/abi-004-kitty-source/kitty-0.45.0')
from kitty_tests import BaseTest, parse_bytes
fixture=Path('/tmp/abi-004-kitty-frames-quiet/prefix.bin').read_bytes()+Path('/tmp/abi-004-kitty-frames-quiet/0.bin').read_bytes()
expected='1d0c0a26ec059fa2d7d813ecbe3f7e10184b20aeb44b544b54590fbad3595ea4'
results=[]
def trial(name,chunks):
    t=BaseTest();screen=t.create_screen(cols=88,lines=33,cell_width=9,cell_height=18)
    for data in chunks: parse_bytes(screen,data)
    count=screen.grman.image_count
    image=screen.grman.image_for_client_id(1) if count else None
    actual=hashlib.sha256(image['data']).hexdigest() if image else None
    row={'case':name,'image_count':count,'sha256':actual,'refs':image['refs.count'] if image else None}
    row['pass']=count==1 and actual==expected and row['refs']==1
    results.append(row)
for size in [1,2,3,4,7,63,64,255,1023,1024,2047,2048,2049,4095,4096,4097,8191,8192,16384,len(fixture)]:
    trial('chunk-'+str(size),[fixture[i:i+size] for i in range(0,len(fixture),size)])
for seed in range(50):
    rng=random.Random(seed); offset=0; chunks=[]
    while offset<len(fixture):
        end=offset+rng.randint(1,8192);chunks.append(fixture[offset:end]);offset=end
    trial('seed-'+str(seed),chunks)
Path('/tmp/abi-004-kitty-parser-results.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps({'cases':len(results),'failed':[r for r in results if not r['pass']]}))
assert all(r['pass'] for r in results)
