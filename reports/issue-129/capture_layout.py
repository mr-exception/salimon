#!/usr/bin/env python3
"""Reproduce issue #129 Linux native layout evidence on a dedicated X11 display."""
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
from salimon_test import Game
from packaged_smoke import input_key


def command(*args):
    return subprocess.check_output(args, text=True).strip()


def capture(window, name):
    time.sleep(1)
    subprocess.run(['import', '-window', window, str(Path(__file__).parent / f'{name}.png')], check=True)


def main():
    os.chdir(ROOT)
    binary = str(ROOT / 'artifacts/build/release/salimon-client')
    setup = {'scenario': 'landed-earth', 'seed': 0, 'step_ms': 16}
    for dpi in ('1', '2'):
        os.environ['WINIT_X11_SCALE_FACTOR'] = dpi
        with Game([binary], setup, time.monotonic() + 60) as game:
            window = command('xdotool', 'search', '--onlyvisible', '--pid', str(game.process.pid)).splitlines()[-1]
            if dpi == '1':
                capture(window, 'no-selection')
                for slot in (1, 2, 5):
                    game.request({'op': 'key', 'key': f'slot_{slot}', 'pressed': True})
                    game.request({'op': 'key', 'key': f'slot_{slot}', 'pressed': False})
                    assert game.request({'op': 'inspect'})['equipment']['selected_slot'] == slot
                    capture(window, f'selected-slot-{slot}')
            game.request({'op': 'key', 'key': 'slot_1', 'pressed': True})
            game.request({'op': 'key', 'key': 'slot_1', 'pressed': False})
            if dpi == '2':
                capture(window, 'dpi-2')
                continue
            game.request({'op': 'look', 'dx': 350, 'dy': 0})
            game.request({'op': 'interact'})
            assert game.request({'op': 'inspect'})['player']['location'] == 'Cockpit'
            capture(window, 'global-message-stacking')
            # A closed-door attempt while flying produces screen-space transient feedback.
            game.request({'op': 'landing'})
            game.request({'op': 'step', 'frames': 250})
            game.request({'op': 'interact'})
            game.request({'op': 'look', 'dx': -350, 'dy': 0})
            for key, frames in [('backward', 110), ('right', 36)]:
                game.request({'op':'key','key':key,'pressed':True})
                game.request({'op':'step','frames':frames})
                game.request({'op':'key','key':key,'pressed':False})
            game.request({'op': 'interact'})
            capture(window, 'transient-message-stacking')
            input_key(game.process.pid, 'F2')
            capture(window, 'precision-tour-hidden')
            input_key(game.process.pid, 'F2')
            command('xdotool', 'windowsize', window, '800', '600')
            capture(window, 'resize-800x600')
    print('Native toolbar layout captures completed.')


if __name__ == '__main__':
    main()
