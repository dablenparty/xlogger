import csv
import os
import re
import time

from inputs import get_gamepad, rescan_devices, UnpluggedError

from src.constants import BUTTON_NAME_MAP, CONTROLLER_EVENTS
from src.utils import get_file_safe_date_string
from src.visualize import visualize_data


def create_data_file():
    """
    Creates a data file with the current date and time as the name.

    :return: Path to the data file
    """
    file_safe_date_string = get_file_safe_date_string()
    file = os.path.join(os.path.dirname(__file__), "data", f'{file_safe_date_string}.csv')
    os.makedirs(os.path.dirname(file), exist_ok=True)  # make sure the data directory exists
    return file


def main(output_file):
    # represents if a button is pressed or not
    press_times = {key: {"time": -1, "state": 0.0} for key in CONTROLLER_EVENTS}

    write_header = not os.path.exists(output_file)

    with open(output_file, "w", newline='') as csvfile:
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
                if event_code == 'SYN_REPORT':
                    continue
                elif re.match(r"^ABS_R?[XY]$", event_code):
                    # skip the stick events for now
                    continue
                now = time.time()
                last_state = press_times.get(event_code, {"time": -1, "state": 0.0})
                down_time = last_state.get("time", -1)
                if event.state != 0:
                    # event state is 0 when the button is not pressed
                    if down_time == -1:
                        # if the button is pressed for the first time, set the down time
                        last_state["time"] = now
                        last_state["state"] = event.state
                elif down_time != -1:
                    # the state isn't 0 and has been pressed before
                    release_time = now
                    press_state = last_state.get("state", 1)
                    last_state["time"] = -1
                    last_state["state"] = event.state
                    key = event_code
                    # dpad events are represented by their axis, the state is the direction
                    if "HAT0" in key:
                        key += f"_{press_state}"
                    button_name = BUTTON_NAME_MAP.get(key, event_code)
                    csv_writer.writerow([down_time, release_time, button_name])
                    print(f"{button_name} held for {release_time - down_time} seconds")


if __name__ == '__main__':
    data_file = create_data_file()
    try:
        main(data_file)
    except KeyboardInterrupt:
        visualize_data(data_file)
