#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod dao {
    use ink::storage::Mapping;
    use openbrush::contracts::traits::psp22::*;
    use scale::{
        Decode,
        Encode,
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
        InvalidVoteType,
        ProposalNotFound,
        ProposalAlreadyExecuted,
        VotePeriodEnded,
        QuorumNotReached,
        ProposalNotAccepted,
        QuorumInvalid,
        AlreadyVoted
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
            //unimplemented!();
            // if quorum < 1 || quorum > 10 {
            //     return Err(DaoError::QuorumInvalid);
            // }
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
                vote_end: start + duration * 60,
                executed: false,
                amount,
            };
            self.next_proposal_id = self.next_proposal_id + 1;
            self.proposals.insert(self.next_proposal_id, &proposal);
            Ok(())
        }

        #[ink(message)]
        pub fn vote(
            &mut self,
            proposal_id: ProposalId,
            vote: VoteType,
        ) -> Result<(), DaoError> {
            if self.check_vote_correctness(vote) == false {
                return Err(DaoError::InvalidVoteType)
            }

            if self.proposals.contains(&proposal_id) == false {
                return Err(DaoError::ProposalNotFound)
            }

            let Some(proposal) = self.proposals.get(&proposal_id);

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

            let votes_distribution = self
                .proposal_votes
                .get(proposal)
                .get_or_insert_with(|| ProposalVote {
                    for_votes: 0,
                    against_votes: 0
                });

            match vote {
                VoteType::Against => {
                    votes_distribution.against_votes = votes_distribution.against_votes + weight;
                },
                VoteType::For => {
                    votes_distribution.for_votes = votes_distribution.for_votes + weight;
                }
            }
            Ok(())
        }

        fn check_vote_correctness(&self, value: VoteType) -> bool {
            match value {
                VoteType::Against | VoteType::For => true,
                _ => false,
            }
        }

        fn get_total_supply(&self) -> Balance {
            let contract_instance = Psp22::from_account_id(self.governance_token);
            contract_instance.total_supply()
        }

        fn get_balance_of(&self, account: AccountId) -> Balance {
            let contract_instance = Psp22::from_account_id(self.governance_token);
            return contract_instance.balance_of(account);
        }

        #[ink(message)]
        pub fn execute(&mut self, proposal_id: ProposalId) -> Result<(), DaoError> {
            if self.proposals.contains(&proposal_id) == false {
                return Err(DaoError::ProposalNotFound)
            }

            let Some(proposal) = self.proposals.get(&proposal_id);

            if proposal.executed == true {
                return Err(DaoError::ProposalAlreadyExecuted);
            }

            let Some(votes_distribution) = self
                .proposal_votes
                .get(proposal);
            
            if votes_distribution.against_votes + votes_distribution.for_votes < self.quorum.into() {
                return Err(DaoError::QuorumNotReached);
            }

            if votes_distribution.against_votes < votes_distribution.for_votes {
                return Err(DaoError::ProposalNotAccepted);
            }

            proposal.executed = true;
            self.transfer(proposal.to, proposal.amount);
            Ok(())
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
            let proposal = governor.get_proposal(0).unwrap();
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
            let execute = governor.execute(0);
            assert_eq!(execute, Err(DaoError::QuorumNotReached));
        }
    }
}
