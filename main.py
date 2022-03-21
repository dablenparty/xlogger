import csv
import os.path
import time

from inputs import get_gamepad, UnpluggedError

BUTTON_MAP = {
    'BTN_NORTH': 'Y',
    'BTN_SOUTH': 'A',
    'BTN_EAST': 'B',
    'BTN_WEST': 'X',
    'BTN_TL': 'LB',
    'BTN_TR': 'RB',
    'BTN_THUMBL': 'L_THUMB',
    'BTN_THUMBR': 'R_THUMB',
    'BTN_SELECT': 'BACK',
    'BTN_START': 'START',
    'ABS_RZ': 'RT',
    'ABS_Z': 'LT',
    'ABS_HAT0X_1': 'DPAD_RIGHT',
    'ABS_HAT0X_-1': 'DPAD_LEFT',
    'ABS_HAT0Y_1': 'DPAD_DOWN',
    'ABS_HAT0Y_-1': 'DPAD_UP',
    'ABS_X': 'L_STICK_X',
    'ABS_Y': 'L_STICK_Y',
    'ABS_RX': 'R_STICK_X',
    'ABS_RY': 'R_STICK_Y',
}

JOYSTICK_CUTOFF = 2000

# represents if a button is pressed or not
CONTROLLER_STATE = {key: {"start": -1, "state": 0} for key in BUTTON_MAP.values()}

if __name__ == '__main__':
    data_file = "events.csv"
    write_header = os.path.exists(data_file)
    with open(data_file, "a+") as csvfile:
        csv_writer = csv.writer(csvfile, delimiter=',', quotechar='"', quoting=csv.QUOTE_MINIMAL)
        # Write header
        if write_header:
            csv_writer.writerow(['time', 'event', 'hold_duration', 'value'])
        while True:
            try:
                events = get_gamepad()
            except UnpluggedError:
                print("\rController unplugged", sep='', end='', flush=True)
                time.sleep(0.5)
                continue
            for event in events:
                if event.code == 'SYN_REPORT':
                    continue
                code = event.code
                event_state = event.state
                if 'HAT0' in event.code:
                    code += f"_{event_state}"
                converted = BUTTON_MAP.get(code, event.code)
                # for now, just ignore the stick events
                if "STICK" in converted:
                    continue
                # if "STICK" in converted and -JOYSTICK_CUTOFF <= event.state <= JOYSTICK_CUTOFF:
                #     continue
                current_state = CONTROLLER_STATE.get(converted)
                print(f"\n{converted} {event_state}")
                if not current_state:
                    continue
                if event_state:
                    if current_state["start"] == -1:
                        current_state["start"] = time.time()
                    current_state["state"] = event_state
                else:
                    if current_state and current_state["start"] != -1:
                        hold_duration = time.time() - current_state["start"]
                        csv_writer.writerow([time.time(), converted, hold_duration, event_state])
                        current_state["start"] = -1
                        current_state["state"] = event_state
