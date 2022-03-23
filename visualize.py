import csv
import pandas as pd
import plotly.express as px


def clean_data():
    with open('./controller.csv', 'r') as csvfile:
        reader = csv.reader(csvfile)
        header = next(reader)
        with open('./cleaned.csv', 'w', newline='') as clean_csvfile:
            writer = csv.writer(clean_csvfile)
            header[0] = 'down_time'
            header[2] = 'up_time'
            header = header[:3]
            writer.writerow(header)
            for row in reader:
                if not row:
                    continue
                row[2] = str(float(row[0]) + float(row[2]))
                writer.writerow(row[:3])


if __name__ == '__main__':
    clean_data()
    df = pd.read_csv('./cleaned.csv')
    # print(df.head())
    df['hold_duration'] = round(df['up_time'] - df['down_time'], 3)
    df['down_time'] = pd.to_datetime(df['down_time'], unit='s')
    df['up_time'] = pd.to_datetime(df['up_time'], unit='s')
    print(df.head())
    fig = px.timeline(df, x_start='down_time', x_end='up_time', y='event', color='event', title='Event Timeline',
                      hover_name='event', custom_data=['event', 'hold_duration'])
    fig.update_traces(hovertemplate="<br>".join([
        "%{customdata[0]}",
        "Duration: %{customdata[1]}s",
    ]))
    fig.show()
