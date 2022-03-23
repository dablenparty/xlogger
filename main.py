import csv
import datetime as dt
import os
import time

from inputs import get_gamepad, rescan_devices, UnpluggedError

from src.constants import BUTTON_MAP


def main():
    # represents if a button is pressed or not
    controller_state = {key: {"start": -1, "state": 0} for key in BUTTON_MAP.values()}

    file_safe_date_string = str(dt.datetime.now()).replace(":", "_")
    data_file = os.path.join(os.path.dirname(__file__), "data", f'{file_safe_date_string}.csv')
    os.makedirs(os.path.dirname(data_file), exist_ok=True)  # make sure the data directory exists
    write_header = not os.path.exists(data_file)

    last_dpad_code = None
    with open(data_file, "w", newline='') as csvfile:
        csv_writer = csv.writer(csvfile, delimiter=',', quotechar='"', quoting=csv.QUOTE_MINIMAL)
        # Write header
        if write_header:
            csv_writer.writerow(['time', 'event', 'hold_duration', 'value'])
        while True:
            try:
                events = get_gamepad()
            except UnpluggedError:
                # only check for new devices every half second
                time.sleep(0.5)
                rescan_devices()
                continue
            for event in events:
                code = event.code
                if code == 'SYN_REPORT':
                    continue
                event_state = event.state
                if 'HAT0' in code:
                    if event_state:
                        code += f"_{event_state}"
                        last_dpad_code = code
                    else:
                        code = last_dpad_code
                converted = BUTTON_MAP.get(code, event.code)
                # for now, just ignore the stick events
                if "STICK" in converted:
                    continue
                current_state = controller_state.get(converted)
                print(f"{converted} {event_state}")
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


if __name__ == '__main__':
    main()
