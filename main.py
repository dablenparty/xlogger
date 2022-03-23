import csv
import os
import time

from inputs import get_gamepad, rescan_devices, UnpluggedError

from src.constants import BUTTON_MAP
from src.utils import get_file_safe_date_string


def create_data_file():
    """
    Creates a data file with the current date and time as the name.

    :return: Path to the data file
    """
    file_safe_date_string = get_file_safe_date_string()
    data_file = os.path.join(os.path.dirname(__file__), "data", f'{file_safe_date_string}.csv')
    os.makedirs(os.path.dirname(data_file), exist_ok=True)  # make sure the data directory exists
    return data_file


def main():
    # represents if a button is pressed or not
    press_times = {key: -1 for key in BUTTON_MAP.values()}
    last_dpad_code = None

    data_file = create_data_file()
    write_header = not os.path.exists(data_file)

    with open(data_file, "w", newline='') as csvfile:
        csv_writer = csv.writer(csvfile, delimiter=',', quotechar='"', quoting=csv.QUOTE_MINIMAL)
        # Write header
        if write_header:
            csv_writer.writerow(['Press Time', 'Release Time', 'Button'])
        while True:
            # tries to get the next event from the gamepad
            try:
                events = get_gamepad()
            except UnpluggedError:
                # rescan for gamepads every half second if no gamepad is found
                time.sleep(0.5)
                rescan_devices()
                continue
            # events is a generator
            # think of this loop like an event emitter and each iteration is the callback for a single event
            for event in events:
                event_code = event.code
                # skips the synchronize reports
                if event_code == 'SYN_REPORT':
                    continue
                event_state = event.state

                # D-Pad is a special case
                if 'HAT0' in event_code:
                    if event_state:
                        event_code += f"_{event_state}"
                        last_dpad_code = event_code
                    else:
                        event_code = last_dpad_code
                button_name = BUTTON_MAP.get(event_code, event.code)

                # TODO: use a separate file for the stick events
                # for now, just ignore the stick events
                if "STICK" in button_name:
                    continue

                # get the last time the button was pressed
                press_time = press_times.get(button_name, -1)

                now = time.time()
                if event_state:
                    # button was pressed
                    if press_time == -1:
                        press_times[button_name] = now
                elif press_time != -1:
                    # button was released
                    print(f"{button_name} released after {round(now - press_time, 4)} seconds")
                    csv_writer.writerow([press_time, now, button_name])
                    press_times[button_name] = -1


if __name__ == '__main__':
    main()
