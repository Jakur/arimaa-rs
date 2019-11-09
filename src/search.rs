use crate::position;
use mcts::transposition_table::*;
use mcts::tree_policy::*;
use mcts::*;


pub fn simple_search(game: ArimaaGame) -> Vec<position::Step> {
    let mut mcts = MCTSManager::new(
        game,
        MyMCTS,
        ArimaaEvaluator,
        UCTPolicy::new(0.5),
        ApproxTable::new(1024),
    );
    mcts.playout_n_parallel(100000, 4);
    mcts.principal_variation(4)
}

#[derive(Default)]
struct MyMCTS;

impl MCTS for MyMCTS {
    type State = ArimaaGame;
    type Eval = ArimaaEvaluator;
    type NodeData = ();
    type ExtraThreadData = ();
    type TreePolicy = UCTPolicy;
    type TranspositionTable = ApproxTable<Self>;

    fn cycle_behaviour(&self) -> CycleBehaviour<Self> {
        CycleBehaviour::UseCurrentEvalWhenCycleDetected
    }
}

#[derive(Clone)]
pub struct ArimaaGame {
    pub pos: position::Position,
    pub end_state: position::EndState,
}

impl ArimaaGame {
    pub fn new(pos: position::Position) -> ArimaaGame {
        ArimaaGame {
            pos,
            end_state: position::EndState::Neither,
        }
    }
}

impl GameState for ArimaaGame {
    type Move = position::Step;
    type Player = position::Side;
    type MoveList = Vec<position::Step>;

    fn current_player(&self) -> Self::Player {
        if self.pos.plies < 8 {
            position::Side::White
        } else if self.pos.plies < 16 {
            position::Side::White
        } else {
            self.pos.side
        }
    }

    fn available_moves(&self) -> Self::MoveList {
        match self.end_state {
            position::EndState::Neither => self.pos.gen_steps(),
            _ => vec![],
        }
    }

    fn make_move(&mut self, mov: &Self::Move) {
        // Slightly bad style, but fine until the final design
        let res = self.pos.do_step(*mov);
        self.end_state = res;
    }
}

impl TranspositionHash for ArimaaGame {
    fn hash(&self) -> u64 {
        self.pos.current_hash
    }
}

struct ArimaaEvaluator;

impl Evaluator<MyMCTS> for ArimaaEvaluator {
    type StateEvaluation = i64;

    fn evaluate_new_state(
        &self,
        state: &ArimaaGame,
        moves: &Vec<position::Step>,
        _: Option<SearchHandle<MyMCTS>>,
    ) -> (Vec<()>, i64) {
        let eval = match state.end_state {
            position::EndState::WhiteWin => 1,
            position::EndState::BlackWin => -1,
            position::EndState::Neither => 0,
        };
        (vec![(); moves.len()], eval)
    }
    fn interpret_evaluation_for_player(&self, evaln: &i64, player: &position::Side) -> i64 {
        match player {
            position::Side::White => *evaln,
            position::Side::Black => -*evaln,
        }
    }
    fn evaluate_existing_state(&self, _: &ArimaaGame, evaln: &i64, _: SearchHandle<MyMCTS>) -> i64 {
        *evaln
    }
}
