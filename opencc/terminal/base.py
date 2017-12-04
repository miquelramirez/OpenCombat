# coding: utf-8
from opencc.simulation.event import NewVisibleOpponent
from opencc.simulation.event import FireEvent
from opencc.simulation.event import DieEvent
from opencc.simulation.event import NoLongerVisibleOpponent
from opencc.simulation.physics import TilePhysics
from opencc.simulation.subject import TileSubject as ManSubject
from opencc.gui.actor import Man as ManActor
from synergine2_cocos2d.terminal import GameTerminal
from synergine2_cocos2d.util import get_map_file_path_from_dir
from synergine2_xyz.move.simulation import FinishMoveEvent
from synergine2_xyz.move.simulation import StartMoveEvent


class CocosTerminal(GameTerminal):
    subscribed_events = [
        FinishMoveEvent,
        StartMoveEvent,
        NewVisibleOpponent,
        NoLongerVisibleOpponent,
        FireEvent,
        DieEvent,
    ]

    def __init__(self, *args, asynchronous: bool, map_dir_path: str, **kwargs):
        super().__init__(*args, **kwargs)
        self.asynchronous = asynchronous
        map_file_path = get_map_file_path_from_dir(map_dir_path)
        self.physics = TilePhysics(
            self.config,
            map_file_path=map_file_path,
        )
        self.map_dir_path = map_dir_path

    def run(self):
        from opencc.gui.base import Game
        from synergine2_cocos2d.gui import SubjectMapper

        self.gui = Game(
            self.config,
            self.logger,
            self,
            physics=self.physics,
            map_dir_path=self.map_dir_path,
        )

        # TODO: Defind on some other place ?
        self.gui.subject_mapper_factory.register_mapper(
            ManSubject,
            SubjectMapper(ManActor),
        )

        self.gui.run()
