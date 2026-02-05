# TAT-DIST-001: Ternary Reward Distributor Contract
# Algorand Smart Contract using PyTEAL
# Version: 1.0.0

from pyteal import *

def approval_program():
    """
    Ternary Reward Distributor Contract (TAT-DIST-001)
    
    Distributes rewards based on witnessed operations from Hedera HCS.
    Supports ternary-weighted distribution with compression efficiency bonuses.
    """
    
    # Global state keys
    admin_key = Bytes("admin")
    oracle_key = Bytes("oracle")
    total_distributed_key = Bytes("total_distributed")
    distribution_count_key = Bytes("distribution_count")
    efficiency_bonus_rate_key = Bytes("efficiency_bonus_rate")
    
    # Local state keys (per user)
    user_rewards_key = Bytes("rewards")
    user_operations_key = Bytes("operations")
    user_last_claim_key = Bytes("last_claim")
    
    # Operations
    op_set_oracle = Bytes("set_oracle")
    op_record_operation = Bytes("record_operation")
    op_distribute_rewards = Bytes("distribute_rewards")
    op_claim_rewards = Bytes("claim_rewards")
    op_update_efficiency_rate = Bytes("update_efficiency_rate")
    
    @Subroutine(TealType.uint64)
    def init():
        return Seq([
            App.globalPut(admin_key, Txn.sender()),
            App.globalPut(oracle_key, Global.zero_address()),
            App.globalPut(total_distributed_key, Int(0)),
            App.globalPut(distribution_count_key, Int(0)),
            App.globalPut(efficiency_bonus_rate_key, Int(5850)),  # 58.50% efficiency bonus (in basis points)
            Return(Int(1))
        ])
    
    @Subroutine(TealType.uint64)
    def is_admin():
        return Txn.sender() == App.globalGet(admin_key)
    
    @Subroutine(TealType.uint64)
    def is_oracle():
        return Txn.sender() == App.globalGet(oracle_key)
    
    @Subroutine(TealType.uint64)
    def set_oracle():
        new_oracle = Txn.accounts[1]
        return Seq([
            Assert(is_admin()),
            App.globalPut(oracle_key, new_oracle),
            Return(Int(1))
        ])
    
    @Subroutine(TealType.uint64)
    def record_operation():
        user = Txn.accounts[1]
        operation_value = Btoi(Txn.application_args[1])
        ternary_efficiency = Btoi(Txn.application_args[2])
        
        # Calculate reward with ternary efficiency bonus
        efficiency_bonus = (operation_value * App.globalGet(efficiency_bonus_rate_key)) / Int(10000)
        base_reward = operation_value + efficiency_bonus
        
        return Seq([
            Assert(Or(is_admin(), is_oracle())),
            App.localPut(
                user,
                user_rewards_key,
                App.localGet(user, user_rewards_key) + base_reward
            ),
            App.localPut(
                user,
                user_operations_key,
                App.localGet(user, user_operations_key) + Int(1)
            ),
            App.globalPut(
                distribution_count_key,
                App.globalGet(distribution_count_key) + Int(1)
            ),
            Log(Concat(Bytes("OPERATION_RECORDED:"), Itob(base_reward))),
            Return(Int(1))
        ])
    
    @Subroutine(TealType.uint64)
    def claim_rewards():
        user_rewards = App.localGet(Txn.sender(), user_rewards_key)
        
        return Seq([
            Assert(user_rewards > Int(0)),
            App.localPut(Txn.sender(), user_rewards_key, Int(0)),
            App.localPut(Txn.sender(), user_last_claim_key, Global.latest_timestamp()),
            App.globalPut(
                total_distributed_key,
                App.globalGet(total_distributed_key) + user_rewards
            ),
            Log(Concat(Bytes("REWARDS_CLAIMED:"), Itob(user_rewards))),
            Return(Int(1))
        ])
    
    @Subroutine(TealType.uint64)
    def update_efficiency_rate():
        new_rate = Btoi(Txn.application_args[1])
        return Seq([
            Assert(is_admin()),
            Assert(new_rate <= Int(10000)),  # Max 100%
            App.globalPut(efficiency_bonus_rate_key, new_rate),
            Return(Int(1))
        ])
    
    # Main router
    program = Cond(
        [Txn.application_id() == Int(0), init()],
        [Txn.on_completion() == OnComplete.OptIn, Return(Int(1))],
        [Txn.on_completion() == OnComplete.CloseOut, Return(Int(1))],
        [Txn.on_completion() == OnComplete.DeleteApplication, Return(is_admin())],
        [Txn.application_args[0] == op_set_oracle, set_oracle()],
        [Txn.application_args[0] == op_record_operation, record_operation()],
        [Txn.application_args[0] == op_claim_rewards, claim_rewards()],
        [Txn.application_args[0] == op_update_efficiency_rate, update_efficiency_rate()],
    )
    
    return program

def clear_state_program():
    return Return(Int(1))

if __name__ == "__main__":
    from pyteal import compileTeal, Mode
    
    print("// TAT-DIST-001 Approval Program")
    print(compileTeal(approval_program(), mode=Mode.Application, version=8))
    print("\n// TAT-DIST-001 Clear State Program")
    print(compileTeal(clear_state_program(), mode=Mode.Application, version=8))
