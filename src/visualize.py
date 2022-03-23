import pandas as pd
import plotly.express as px


def visualize_data(csv_file):
    df = pd.read_csv(csv_file)
    # print(df.head())
    df['Hold Duration'] = round(df['Release Time'] - df['Press Time'], 3)
    df['Press Time'] = pd.to_datetime(df['Press Time'], unit='s')
    df['Release Time'] = pd.to_datetime(df['Release Time'], unit='s')
    print(df.head())
    fig = px.timeline(df, x_start='Press Time', x_end='Release Time', y='Button', color='Button',
                      title='Button Timeline',
                      hover_name='Button', custom_data=['Button', 'Hold Duration'])
    fig.update_traces(hovertemplate="<br>".join([
        "%{customdata[0]}",
        "Duration: %{customdata[1]}s",
    ]))
    fig.show()
