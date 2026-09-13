import runpy,os,termios,types,tempfile
from pathlib import Path
m=runpy.run_path('scripts/check-api-entry-points.py')
t=m['Terminal'].__new__(m['Terminal'])
t.master,writer=os.pipe();os.close(writer)
spare,t.slave=os.openpty()
t.original=termios.tcgetattr(t.slave)
t.child=types.SimpleNamespace(poll=lambda:0,returncode=0)
t.output=bytearray(b'ENTRY_POINT_CLEAN_EXIT')
with tempfile.TemporaryDirectory() as d:
 t.capture=Path(d)/'capture.bin'
 t.finish(host_modes=False)
print('EOF_DRAIN_RETURNED',flush=True)
os.close(t.master);os.close(t.slave);os.close(spare)
