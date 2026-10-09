"""Dedicated X11 evidence capture; resize the sole Salimon window and check PNG integrity."""
import os
from pathlib import Path
import struct
import subprocess
import sys
import time
import zlib


def complete_png(path):
    data = Path(path).read_bytes()
    if data[:8] != b'\x89PNG\r\n\x1a\n':
        return False
    offset = 8
    while offset + 12 <= len(data):
        length, kind = struct.unpack_from('>I4s', data, offset)
        end = offset + length + 12
        if end > len(data):
            return False
        checksum = struct.unpack_from('>I', data, end - 4)[0]
        if zlib.crc32(data[offset + 4:end - 4]) != checksum:
            return False
        if kind == b'IEND':
            return length == 0 and end == len(data)
        offset = end
    return False


def main():
    if len(sys.argv) != 2:
        raise SystemExit('usage: capture_resized.py OUTPUT.png')
    windows = subprocess.check_output(['xdotool', 'search', '--onlyvisible', '--name', 'Salimon']).decode().split()
    if len(windows) != 1:
        raise RuntimeError(f'expected one Salimon window, found {len(windows)}')
    width, height = os.environ.get('SALIMON_CAPTURE_SIZE', '1280x800').split('x')
    subprocess.run(['xdotool', 'windowsize', windows[0], width, height], check=True)
    time.sleep(.7)
    for _ in range(3):
        subprocess.run(['import', '-window', windows[0], sys.argv[1]], check=True)
        if complete_png(sys.argv[1]):
            return
        time.sleep(.2)
    raise RuntimeError('capture produced an incomplete/corrupt PNG')


if __name__ == '__main__':
    main()
