#![cfg_attr(not(feature = "std"), no_std, no_main)]

// use my_psp22_metadata::GovernanceTokenRef;

#[ink::contract]
pub mod dao {
    use ink::storage::Mapping;
    use openbrush::contracts::traits::psp22::*;
    use scale::{
        Decode,
        Encode,
    };
    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        DefaultEnvironment,
    };

    #[derive(Encode, Decode)]
    #[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq, scale_info::TypeInfo))]
    pub enum VoteType {
        Against,
        For,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum DaoError { 
        // to implement
        AmountExceedsBalance,
        AmountShouldNotBeZero,
        DurationError,
        ProposalNotFound,
        ProposalAlreadyExecuted,
        VotePeriodEnded,
        QuorumNotReached,
        ProposalNotAccepted,
        QuorumInvalid,
        AlreadyVoted,
        FailedTransfer,
    }

    #[derive(Encode, Decode)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink::storage::traits::StorageLayout
        )
    )]
    pub struct Proposal {
        to: AccountId,
        vote_start: u64,
        vote_end: u64,
        executed: bool,
        amount: u128,
    }

    #[derive(Encode, Decode, Default)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink::storage::traits::StorageLayout
        )
    )]
    pub struct ProposalVote {
        // to implement
        for_votes: u128,
        against_votes: u128,
    }

    pub type ProposalId = u64;
    static ONE_MINUTE: u64 = 60;

    #[ink(storage)]
    pub struct Governor {
        // to implement
        proposals: Mapping<ProposalId, Proposal>,
        next_proposal_id: ProposalId,
        votes: Mapping<(ProposalId, AccountId), ()>,
        proposal_votes: Mapping<Proposal, ProposalVote>,
        governance_token: AccountId,
        quorum: u8,
    }

    impl Governor {
        #[ink(constructor, payable)]
        pub fn new(governance_token: AccountId, quorum: u8) -> Self {
            Self {
                proposals: Default::default(),
                next_proposal_id: Default::default(),
                votes: Default::default(),
                proposal_votes: Default::default(),
                governance_token,
                quorum,
            }
        }

        #[ink(message)]
        pub fn propose(
            &mut self,
            to: AccountId,
            amount: Balance,
            duration: u64, 
        ) -> Result<(), DaoError> {
            if amount == 0 {
                return Err(DaoError::AmountShouldNotBeZero)
            }

            if duration == 0 {
                return Err(DaoError::DurationError);
            }
            
            if amount > self.env().balance() {
                return Err(DaoError::AmountExceedsBalance);
            }

            let start = self.env().block_timestamp();
            let proposal = Proposal {
                to,
                vote_start: start,
                vote_end: start + duration * ONE_MINUTE,
                executed: false,
                amount,
            };
            
            self.next_proposal_id = self.next_proposal_id() + 1;
            self.proposals.insert(self.next_proposal_id, &proposal);
            self.proposal_votes.insert(proposal, &{ ProposalVote {
                    for_votes: 0,
                    against_votes: 0
                }});
            Ok(())
        }

        #[ink(message)]
        pub fn vote(
            &mut self,
            proposal_id: ProposalId,
            vote: VoteType,
        ) -> Result<(), DaoError> {
            if self.proposals.contains(&proposal_id) == false {
                return Err(DaoError::ProposalNotFound)
            }

            let proposal = self.get_proposal(proposal_id).unwrap();

            if proposal.executed == true {
                return Err(DaoError::ProposalAlreadyExecuted);
            }

            if proposal.vote_end < self.env().block_timestamp() {
                return Err(DaoError::VotePeriodEnded);
            }

            let caller = self.env().caller();

            if self.votes.contains(&(proposal_id, caller)) {
                return Err(DaoError::AlreadyVoted);
            }

            self.votes.insert(&(proposal_id, caller),&());

            let total_supply = self.get_total_supply();
            let caller_balance = self.get_balance_of(caller);
            let weight = caller_balance/total_supply * 100; //in percentage

            let _votes_distribution = self
                .proposal_votes
                .get(&proposal);

            match vote {
                VoteType::Against => {
                    let Some(mut votes_distribution) = self
                        .proposal_votes
                        .get(proposal) else { todo!() };
                    votes_distribution.against_votes = votes_distribution.against_votes + weight;
                },
                VoteType::For => {
                    let Some(mut votes_distribution) = self
                        .proposal_votes
                        .get(proposal) else { todo!() };
                    votes_distribution.for_votes = votes_distribution.for_votes + weight;
                }
            }
            Ok(())
        }

        fn get_proposal(&self, proposal_id: ProposalId) -> Option<Proposal> {
            if let Some(proposal) = self.proposals.get(&proposal_id) {
                Some(proposal)
            } else {
                None
            }
        }

        fn get_proposal_votes(&self, proposal_id: ProposalId) -> Option<ProposalVote> {
            let proposal = self.get_proposal(proposal_id).unwrap();
            if let Some(votes_distribution) = self.proposal_votes.get(&proposal) {
                Some(votes_distribution)
            } else {
                None
            }
        }

        fn next_proposal_id(&self) -> ProposalId {
            // self.next_proposal_id = self.next_proposal_id + 1;
            self.next_proposal_id
        }

        fn get_total_supply(&self) -> Balance {
            let total_supply = build_call::<DefaultEnvironment>()
                .call(self.governance_token)
                .gas_limit(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("total_supply")))
                )
                .returns::<Balance>()
                .invoke();
            total_supply
        }

        fn get_balance_of(&self, account: AccountId) -> Balance {
            let balance = build_call::<DefaultEnvironment>()
                .call(self.governance_token)
                .gas_limit(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("balance_of")))
                    .push_arg(&account)
                )
                .returns::<Balance>()
                .invoke();
            balance
        }

        #[ink(message)]
        pub fn execute(&mut self, proposal_id: ProposalId) -> Result<(), DaoError> {
            if self.proposals.contains(&proposal_id) == false {
                return Err(DaoError::ProposalNotFound)
            }

            let mut proposal = self.get_proposal(proposal_id).unwrap();

            if proposal.executed == true {
                return Err(DaoError::ProposalAlreadyExecuted);
            }

            let votes_distribution = self.get_proposal_votes(proposal_id).unwrap();
            
            if votes_distribution.against_votes + votes_distribution.for_votes < self.quorum.into() {
                return Err(DaoError::QuorumNotReached);
            }

            if votes_distribution.against_votes < votes_distribution.for_votes {
                return Err(DaoError::ProposalNotAccepted);
            }

            proposal.executed = true;
            
            match self.transfer(proposal.to, proposal.amount) {
                Ok(_) => Ok(()),
                Err(_error) => {
                    Err(DaoError::FailedTransfer)
                }
            }
        }

        fn transfer(&mut self, recipient: AccountId, amount: u128) -> Result<(), ink_env::Error> {
            ink_env::transfer::<ink_env::DefaultEnvironment>(recipient, amount)?;
            Ok(())
        }

        // used for test
        #[ink(message)]
        pub fn now(&self) -> u64 {
            self.env().block_timestamp()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn create_contract(initial_balance: Balance) -> Governor {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), initial_balance);
            Governor::new(AccountId::from([0x01; 32]), 50)
        }

        fn contract_id() -> AccountId {
            ink::env::test::callee::<ink::env::DefaultEnvironment>()
        }

        fn default_accounts(
        ) -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
                account_id, balance,
            )
        }

        #[ink::test]
        fn propose_works() {
            let accounts = default_accounts();
            let mut governor = create_contract(1000);
            assert_eq!(
                governor.propose(accounts.django, 0, 1),
                Err(DaoError::AmountShouldNotBeZero)
            );
            assert_eq!(
                governor.propose(accounts.django, 100, 0),
                Err(DaoError::DurationError)
            );
            let result = governor.propose(accounts.django, 100, 1);
            assert_eq!(result, Ok(()));
            let proposal = governor.get_proposal(1).unwrap();
            let now = governor.now();
            assert_eq!(
                proposal,
                Proposal {
                    to: accounts.django,
                    amount: 100,
                    vote_start: 0,
                    vote_end: now + 1 * ONE_MINUTE,
                    executed: false,
                }
            );
            assert_eq!(governor.next_proposal_id(), 1);
        }

        #[ink::test]
        fn quorum_not_reached() {
            let mut governor = create_contract(1000);
            let result = governor.propose(AccountId::from([0x02; 32]), 100, 1);
            assert_eq!(result, Ok(()));
            assert_eq!(governor.next_proposal_id(), 1);
            let execute = governor.execute(1);
            assert_eq!(execute, Err(DaoError::QuorumNotReached));
        }
    }
}
