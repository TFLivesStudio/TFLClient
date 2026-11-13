"""CI diagnostic only: resolve native imports without running the test executable."""
import ctypes
import glob
import pathlib
import pefile

kernel = ctypes.WinDLL('kernel32', use_last_error=True)
kernel.GetProcAddress.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
kernel.GetProcAddress.restype = ctypes.c_void_p
kernel.GetModuleFileNameW.argtypes = [ctypes.c_void_p, ctypes.c_wchar_p, ctypes.c_uint]
for path in glob.glob('target/debug/deps/tflclient_lib-*.exe'):
    print('BINARY', pathlib.Path(path).name, flush=True)
    pe = pefile.PE(path)
    for entry in getattr(pe, 'DIRECTORY_ENTRY_IMPORT', []):
        name = entry.dll.decode()
        try:
            library = ctypes.WinDLL(name)
            loaded = ctypes.create_unicode_buffer(4096)
            kernel.GetModuleFileNameW(library._handle, loaded, len(loaded))
            print('DLL', name, loaded.value, flush=True)
            for symbol in entry.imports:
                if symbol.name and not kernel.GetProcAddress(library._handle, symbol.name):
                    print('MISSING ENTRYPOINT', name, symbol.name.decode(), flush=True)
        except OSError as error:
            print('DLL LOAD FAILED', name, str(error), flush=True)
