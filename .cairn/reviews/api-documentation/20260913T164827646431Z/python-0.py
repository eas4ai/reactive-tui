import ctypes
import os

lib = ctypes.CDLL(os.environ["RTUI_LIBRARY_PATH"])
lib.rtui_init.argtypes = []
lib.rtui_init.restype = ctypes.c_int
lib.rtui_cleanup.argtypes = []
lib.rtui_cleanup.restype = None
