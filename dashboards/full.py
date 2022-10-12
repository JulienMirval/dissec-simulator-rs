import sys
import plotly_express as px
from dash import html, dcc, dash
import numpy as np
import pandas as pd
import json

roles = ["Aggregator", "LeafAggregator", "Contributor", "Backup", "Querier"]
statistics = [
    "initial_nodes",
    "final_nodes",
    "failures",
    "work",
    "messages",
    "work_per_node",
    "delta_nodes",
    "bandwidth",
]


def get_data(path):
    df = pd.read_csv(path, sep=",", decimal=".")
    # We used different decimal format, check which one we have
    if "float64" not in df.dtypes.unique():
        df = pd.read_csv(path, sep=",")

    print(df.columns)

    df["strategy"] = df["failure_handling"]

    df["latency"] = (df["arrival_time"] - df["departure_time"]).apply(
        lambda x: max(0, x)
    )
    df["simulation_length"] = (
        df.groupby("seed")["departure_time"].min()
        + df.groupby("seed")["departure_time"].min()
    )

    return df


def generate_graphs(data):
    graphs = {}

    # Timeline
    graphs["message_timeline_fig"] = px.scatter(
        data,
        x="arrival_time",
        y="receiver_address",
        color="message_type",
        hover_name="message_type",
        hover_data=[
            "arrival_time",
            "departure_time",
            "receiver_address",
            "emitter_address",
            "seed",
        ],
    )
    # graphs["version_timeline_fig"] = px.scatter(
    #     pd.DataFrame(columns=data.columns),
    #     x="arrival_time",
    #     y="currently_circulating_ids",
    #     color="seed",
    #     hover_name="type",
    #     hover_data=["receiver_address", "emitter_address", "seed"],
    # )
    graphs["bandwidth_timeline_fig"] = px.scatter(
        data,
        x="arrival_time",
        y="total_bandwidth",
        color="seed",
        hover_name="message_type",
        hover_data=["receiver_address", "emitter_address", "seed"],
    )
    graphs["work_timeline_fig"] = px.scatter(
        data,
        x="arrival_time",
        y="total_work",
        color="seed",
        hover_name="message_type",
        hover_data=["receiver_address", "emitter_address", "seed"],
    )
    # graphs["messages_timeline_fig"] = px.scatter(
    #     pd.DataFrame(columns=data.columns),
    #     x="arrival_time",
    #     y="messages_total",
    #     color="seed",
    #     hover_name="type",
    #     hover_data=["receiver_address", "emitter_address", "seed"],
    # )
    # graphs["message_stats_fig"] = px.box(
    #     pd.DataFrame(columns=data.columns),
    #     x="type",
    #     y="latency",
    #     hover_name="type",
    #     hover_data=["emitter_address"],
    #     points="all",
    # )

    return html.Div(
        children=[
            dcc.Graph(id="message_timeline", figure=graphs["message_timeline_fig"]),
            # dcc.Graph(id="version_timeline", figure=graphs["version_timeline_fig"]),
            dcc.Graph(id="bandwidth_timeline", figure=graphs["bandwidth_timeline_fig"]),
            dcc.Graph(id="work_timeline", figure=graphs["work_timeline_fig"]),
            # dcc.Graph(id="messages_timeline", figure=graphs["messages_timeline_fig"]),
            # dcc.Graph(id="message_stats", figure=graphs["message_stats_fig"]),
        ]
    )


if __name__ == "__main__":
    print(sys.argv)
    # try:
    data = get_data(sys.argv[1])
    # except:
    #     print("No file argument given")
    #     sys.exit(1)

    seeds = pd.unique(data["seed"])
    strategies = pd.unique(data["strategy"])
    types = pd.unique(data["message_type"])
    data["average_failure_time"] = data["average_failure_time"].round(6)
    failure_probabilities = np.sort(pd.unique(data["average_failure_time"]))

    # Remove strategies not present in the data
    strategies_map = dict(
        EAGER="Eager", OPTI="Optimistic", PESS="Pessimistic", STRAW="Strawman"
    )
    for k in set(strategies_map.keys()).difference(strategies):
        del strategies_map[k]

    grouped = data.groupby(["seed", "strategy"], as_index=False)[
        ["simulation_length", "total_work", "average_failure_time", "completeness",]
    ].max()
    grouped["average_failure_time"] = grouped["average_failure_time"].round(5)

    app = dash.Dash(__name__)

    app.layout = html.Div(
        children=[
            html.H1(
                children=f"Latency vs Reception time",
                style={"textAlign": "center", "color": "#7FDBFF"},
            ),
            #
            # Timeline
            #
            html.H1("Timeline:"),
            html.Div(
                style={"justifyContent": "center"},
                children=[
                    html.H3("Y = ?"),
                    dcc.Dropdown(
                        [
                            {"label": "Receiver", "value": "receiver"},
                            {"label": "Emitter", "value": "emitter"},
                        ],
                        "receiver",
                        id="y-axis",
                    ),
                ],
            ),
            html.Div(
                style={"justifyContent": "center"},
                children=[
                    html.H3("Protocol executions:"),
                    dcc.Checklist(
                        id="runs-list",
                        options=["All"] + [i for i in seeds],
                        value=[],
                        style={
                            "display": "flex",
                            "flex-wrap": "wrap",
                            "flex-direction": "row",
                        },
                        labelStyle={
                            "display": "flex",
                            "direction": "row",
                            "margin": "5px",
                        },
                    ),
                ],
            ),
            html.Div(
                style={"justifyContent": "center"},
                children=[
                    html.H3("Message types:"),
                    dcc.Checklist(
                        id="types-list",
                        options=types,
                        value=types,
                        style={
                            "display": "flex",
                            "flex-wrap": "wrap",
                            "flex-direction": "row",
                        },
                        labelStyle={
                            "display": "flex",
                            "direction": "row",
                            "margin": "5px",
                        },
                    ),
                ],
            ),
            html.Div(
                [
                    html.H3("Theoretical failure rate"),
                    dcc.RangeSlider(
                        0,
                        0.0001,
                        failure_probabilities[1] - failure_probabilities[0]
                        if len(failure_probabilities) > 1
                        else None,
                        value=[0, failure_probabilities[-1]],
                        id="failure-rates-range",
                    ),
                ]
            ),
            html.Div(
                [
                    html.H3("Show latencies"),
                    dcc.Checklist(["YES"], [], id="show-latencies",),
                ]
            ),
            html.Div(id="graphs", children=[]),
        ]
    )

    @app.callback(
        [dash.Output(component_id="graphs", component_property="children"),],
        [
            dash.Input(component_id="y-axis", component_property="value"),
            dash.Input(component_id="runs-list", component_property="value"),
            dash.Input(component_id="types-list", component_property="value"),
            dash.Input(component_id="failure-rates-range", component_property="value"),
            dash.Input(component_id="show-latencies", component_property="value"),
        ],
    )
    def update_timeline(
        selected_y_axis,
        selected_seeds,
        selected_types,
        selected_failures,
        show_latencies,
    ):
        df = data.copy()
        df = df[
            df["average_failure_time"].isin(
                [
                    i
                    for i in failure_probabilities
                    if i <= selected_failures[1] and i >= selected_failures[0]
                ]
            )
        ]
        if "All" not in selected_seeds:
            df = df[df["seed"].isin(selected_seeds)]
        df = df[df["message_type"].isin(selected_types)]

        return [generate_graphs(data)]

    app.run_server(debug=True)
