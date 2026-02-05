# TAT-GOV-001: Ternary Governance Contract
# Algorand Smart Contract using PyTEAL
# Version: 1.0.0

from pyteal import *

def approval_program():
    """
    Ternary Governance Contract (TAT-GOV-001)
    
    Manages governance voting and proposal execution for the Salvi Framework.
    Implements ternary voting logic with A/B/C representation support.
    """
    
    # Global state keys
    admin_key = Bytes("admin")
    proposal_count_key = Bytes("proposal_count")
    quorum_threshold_key = Bytes("quorum_threshold")
    voting_period_key = Bytes("voting_period")
    
    # Proposal state keys (per proposal)
    def proposal_key(proposal_id):
        return Concat(Bytes("proposal_"), Itob(proposal_id))
    
    def votes_for_key(proposal_id):
        return Concat(Bytes("votes_for_"), Itob(proposal_id))
    
    def votes_against_key(proposal_id):
        return Concat(Bytes("votes_against_"), Itob(proposal_id))
    
    def votes_abstain_key(proposal_id):
        return Concat(Bytes("votes_abstain_"), Itob(proposal_id))
    
    # Operations
    op_create_proposal = Bytes("create_proposal")
    op_vote = Bytes("vote")
    op_execute_proposal = Bytes("execute_proposal")
    op_update_quorum = Bytes("update_quorum")
    
    # Initialize contract
    @Subroutine(TealType.uint64)
    def init():
        return Seq([
            App.globalPut(admin_key, Txn.sender()),
            App.globalPut(proposal_count_key, Int(0)),
            App.globalPut(quorum_threshold_key, Int(51)),  # 51% quorum
            App.globalPut(voting_period_key, Int(604800)),  # 7 days in seconds
            Return(Int(1))
        ])
    
    # Create a new proposal
    @Subroutine(TealType.uint64)
    def create_proposal():
        proposal_id = App.globalGet(proposal_count_key) + Int(1)
        return Seq([
            App.globalPut(proposal_count_key, proposal_id),
            App.globalPut(proposal_key(proposal_id), Global.latest_timestamp()),
            App.globalPut(votes_for_key(proposal_id), Int(0)),
            App.globalPut(votes_against_key(proposal_id), Int(0)),
            App.globalPut(votes_abstain_key(proposal_id), Int(0)),
            Return(Int(1))
        ])
    
    # Cast a vote on a proposal
    # Vote types: 1 = For (A), 2 = Against (B), 3 = Abstain (C)
    @Subroutine(TealType.uint64)
    def vote():
        proposal_id = Btoi(Txn.application_args[1])
        vote_type = Btoi(Txn.application_args[2])
        
        return Seq([
            Assert(vote_type >= Int(1)),
            Assert(vote_type <= Int(3)),
            If(vote_type == Int(1),
                App.globalPut(
                    votes_for_key(proposal_id),
                    App.globalGet(votes_for_key(proposal_id)) + Int(1)
                ),
            If(vote_type == Int(2),
                App.globalPut(
                    votes_against_key(proposal_id),
                    App.globalGet(votes_against_key(proposal_id)) + Int(1)
                ),
                App.globalPut(
                    votes_abstain_key(proposal_id),
                    App.globalGet(votes_abstain_key(proposal_id)) + Int(1)
                )
            )),
            Return(Int(1))
        ])
    
    # Execute a passed proposal
    @Subroutine(TealType.uint64)
    def execute_proposal():
        proposal_id = Btoi(Txn.application_args[1])
        votes_for = App.globalGet(votes_for_key(proposal_id))
        votes_against = App.globalGet(votes_against_key(proposal_id))
        total_votes = votes_for + votes_against
        
        return Seq([
            Assert(total_votes > Int(0)),
            Assert((votes_for * Int(100)) / total_votes >= App.globalGet(quorum_threshold_key)),
            Log(Bytes("PROPOSAL_EXECUTED")),
            Return(Int(1))
        ])
    
    # Main router
    program = Cond(
        [Txn.application_id() == Int(0), init()],
        [Txn.on_completion() == OnComplete.OptIn, Return(Int(1))],
        [Txn.on_completion() == OnComplete.CloseOut, Return(Int(1))],
        [Txn.on_completion() == OnComplete.DeleteApplication, Return(Txn.sender() == App.globalGet(admin_key))],
        [Txn.application_args[0] == op_create_proposal, create_proposal()],
        [Txn.application_args[0] == op_vote, vote()],
        [Txn.application_args[0] == op_execute_proposal, execute_proposal()],
    )
    
    return program

def clear_state_program():
    return Return(Int(1))

if __name__ == "__main__":
    from pyteal import compileTeal, Mode
    
    print("// TAT-GOV-001 Approval Program")
    print(compileTeal(approval_program(), mode=Mode.Application, version=8))
    print("\n// TAT-GOV-001 Clear State Program")
    print(compileTeal(clear_state_program(), mode=Mode.Application, version=8))
