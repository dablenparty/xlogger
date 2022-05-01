import pandas as pd
import plotly.express as px
import sys


def visualize_data(csv_file):
    try:
        df = pd.read_csv(csv_file)
    except pd.errors.EmptyDataError:
        print(f'No data found in {csv_file}')
        sys.exit(1)
    # print(df.head())
    df['HoldDuration'] = round(df['ReleaseTime'] - df['PressTime'], 3)
    df['PressTime'] = pd.to_datetime(df['PressTime'], unit='s')
    df['ReleaseTime'] = pd.to_datetime(df['ReleaseTime'], unit='s')
    print(df.head())
    fig = px.timeline(df, x_start='PressTime', x_end='ReleaseTime', y='Button', color='Button',
                      title='Button Timeline',
                      hover_name='Button', custom_data=['Button', 'HoldDuration'])
    fig.update_traces(hovertemplate="<br>".join([
        "%{customdata[0]}",
        "Duration: %{customdata[1]}s",
    ]))
    fig.show()

if __name__ == '__main__':
    visualize_data(sys.argv[1])
