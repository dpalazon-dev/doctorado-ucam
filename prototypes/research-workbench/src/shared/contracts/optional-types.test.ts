import {it,expect} from 'vitest';
import type {PageDto,WorkflowGetPhaseDefinitionArgs,ConceptUpdateArgs} from './generated/contracts';
// Compile-time parity: generated optional numbers cannot accept explicit null.
const page:PageDto<string>={items:[],nextCursor:null};
const numbered:PageDto<string>={...page,total:1};
// @ts-expect-error total is optional, not nullable
const nullPage:PageDto<string>={...page,total:null};
const args:WorkflowGetPhaseDefinitionArgs={requestId:'00000000-0000-4000-8000-000000000001',phaseCode:'PRE'};
// @ts-expect-error version is optional, not nullable
const nullVersion:WorkflowGetPhaseDefinitionArgs={...args,version:null};
const cleared:ConceptUpdateArgs={requestId:args.requestId,conceptId:args.requestId,expectedRevision:0,domain:null};
it('canonical optional number and nullable patch declarations remain distinct',()=>{expect(numbered.total).toBe(1);expect(cleared.domain).toBeNull();expect(nullPage.total).toBeNull();expect(nullVersion.version).toBeNull();});
