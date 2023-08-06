import type {ReturnNumber} from "@727-ventures/typechain-types";
import type * as ReturnTypes from '../types-returns/dao';

export interface ProposalSubmitted {
	proposalId: number;
	to: ReturnTypes.AccountId;
	amount: ReturnNumber;
	voteStart: number;
	voteEnd: number;
	caller: ReturnTypes.AccountId;
}

export interface ProposalClosed {
	proposalId: number;
	to: ReturnTypes.AccountId;
	amount: ReturnNumber;
	caller: ReturnTypes.AccountId;
}

