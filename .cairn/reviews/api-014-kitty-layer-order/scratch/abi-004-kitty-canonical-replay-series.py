from pathlib import Path
import os,subprocess,json
os.chdir('/home/shawn/workspace2/reactive-tui')
env={**os.environ,'PATH':'/tmp/rtui-image-tools/bin:'+os.environ['PATH'],'LP_NUM_THREADS':'8','RAYON_NUM_THREADS':'8','PYTHONDONTWRITEBYTECODE':'1'}
results=[]
for index in range(3):
 directory=Path('/tmp/abi-004-kitty-canonical-replay-'+str(index));directory.mkdir(exist_ok=False)
 with (directory/'driver.out').open('w') as out:
  result=subprocess.run(['/usr/bin/python3','-B','/tmp/abi-004-kitty-canonical-replay-capture.py','kitty',str(directory),'kitty'],env=env,stdout=out,stderr=subprocess.STDOUT,timeout=90)
 data=json.loads((directory/'pixels.json').read_text()) if (directory/'pixels.json').exists() else None
 row={'index':index,'exit':result.returncode,'pixels':data};results.append(row)
 print(json.dumps(row),flush=True)
Path('/tmp/abi-004-kitty-canonical-replay-results.json').write_text(json.dumps(results,indent=2)+'\n')
