from inputs import get_gamepad

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

if __name__ == '__main__':
    while True:
        events = get_gamepad()
        for event in events:
            if event.code != 'SYN_REPORT':
                code = event.code
                if 'HAT0' in event.code:
                    code += f"_{event.state}"
                print(BUTTON_MAP.get(code, event.code), event.state)
