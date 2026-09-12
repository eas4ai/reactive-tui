from pathlib import Path
import os,select,sys,time,tty,termios,json,re
command=Path(sys.argv[1]); output=Path(sys.argv[2])
data=Path('/home/shawn/workspace2/reactive-tui/.cairn/reviews/api-image-hosts/20260912T132044026958Z/kitty-kitty/terminal.bin').read_bytes().replace(b'q=2',b'q=0')
starts=[m.start() for m in re.finditer(rb'\x1b\[2J',data)]
end=data.index(b'\x1b[0m',starts[-1]);frames=[data[starts[i]:starts[i+1] if i<2 else end] for i in range(3)]
saved=termios.tcgetattr(0); responses=[];started=time.monotonic();shown=None
try:
 tty.setraw(0);os.write(1,data[:starts[0]])
 while time.monotonic()-started<45:
  stage=int(command.read_text())
  if stage==3:break
  if shown!=stage:os.write(1,frames[stage]);shown=stage
  if select.select([0],[],[],0.02)[0]:responses.append({'stage':stage,'elapsed':time.monotonic()-started,'reply':repr(os.read(0,8192))})
finally:
 os.write(1,data[end:]);termios.tcsetattr(0,termios.TCSANOW,saved)
 (output/'responses.json').write_text(json.dumps(responses,indent=2)+'\n')
