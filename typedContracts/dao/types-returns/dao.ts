import type BN from 'bn.js';
import type {ReturnNumber} from '@727-ventures/typechain-types';

export type AccountId = string | number[]

export enum LangError {
	couldNotReadInput = 'CouldNotReadInput'
}

export enum DaoError {
	amountExceedsBalance = 'AmountExceedsBalance',
	amountShouldNotBeZero = 'AmountShouldNotBeZero',
	durationError = 'DurationError',
	proposalNotFound = 'ProposalNotFound',
	proposalAlreadyExecuted = 'ProposalAlreadyExecuted',
	votePeriodEnded = 'VotePeriodEnded',
	quorumNotReached = 'QuorumNotReached',
	proposalNotAccepted = 'ProposalNotAccepted',
	quorumInvalid = 'QuorumInvalid',
	alreadyVoted = 'AlreadyVoted',
	failedTransfer = 'FailedTransfer'
}

export enum VoteType {
	against = 'Against',
	for = 'For'
}

