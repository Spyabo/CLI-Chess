from board import Board, draw_board_black, draw_board_white

from textual.app import App, ComposeResult
from textual.containers import Grid
from textual.reactive import reactive
from textual.widget import Widget
from textual.widgets import Input, RichLog
from textual import on, events


class Text_Board(Widget):
    x = Board.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
    board = reactive(draw_board_white(x))

    def render(self) -> str:
        return self.board


class MyApp(App):
    CSS_PATH = "app.tcss"
    chess = Board.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")

    def compose(self) -> ComposeResult:
        yield Grid(
            Text_Board(id="board"),
            RichLog(),
            Input(placeholder="Enter your move", id="input"),
        )

    def on_key(self, event: events.Key) -> None:
        text = self.query_one(Input).value
        if event.key == "enter":
            self.chess.move((0, 1), (0, 3))
            self.query_one(RichLog).write(text)
            self.query_one(Text_Board).board = draw_board_white(self.chess)


if __name__ == "__main__":
    app = MyApp()
    app.run()
