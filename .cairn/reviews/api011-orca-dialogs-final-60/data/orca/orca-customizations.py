from orca import debug
if debug.debugFile is not None:
    debug.debugFile.reconfigure(line_buffering=True)
