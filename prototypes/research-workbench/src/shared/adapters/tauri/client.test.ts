import {it,expect,vi} from 'vitest';
import {createTauriApis} from './client';
import fixture from '../../../../contracts/fixtures/app-info.json';
import preDefinition from '../../../../docs/architecture/phase-definitions/PRE.v1.json';
const requestId=fixture.requestId;
it('received invalid envelopes or payloads are integrity failures without retries or private content',async()=>{
 for(const response of [{...fixture,contractVersion:2},{...fixture,data:{private:'C:/private/source.pdf'}}]){
  const result=await createTauriApis(async()=>response).settings.getAppInfo({requestId});
  expect(result.ok).toBe(false);if(!result.ok){expect(result.error.code).toBe('IntegrityFailure');expect(result.error.retryable).toBe(false);}
  expect(JSON.stringify(result)).not.toContain('private');
 }
});
it('transport failure and structured failures retain their separate semantics',async()=>{
 const thrown=await createTauriApis(async()=>{throw new Error('private');}).settings.getAppInfo({requestId});
 expect(thrown.ok).toBe(false);if(!thrown.ok)expect(thrown.error.code).toBe('StorageUnavailable');
 const failure={contractVersion:1,requestId,ok:false,error:{code:'Busy',message:'Biblioteca ocupada.',retryable:true}};
 for(const invoke of [async()=>failure,async()=>{throw failure;}]) expect(await createTauriApis(invoke).settings.getAppInfo({requestId})).toEqual(failure);
});
it('validates payload and envelope from simulated invoke',async()=>{const invoke=vi.fn(async()=>fixture);const api=createTauriApis(invoke);const result=await api.settings.getAppInfo({requestId});expect(result.ok).toBe(true);expect(invoke).toHaveBeenCalledWith('settings_get_app_info',{args:{requestId}});});
it('routes all eight backend workflow commands while candidate operations stay gated',async()=>{
 const phase={paperId:requestId,code:'PRE',definitionVersion:1,state:'IN_PROGRESS',revision:1,completedAt:null};
 const answer={paperId:requestId,phaseCode:'PRE',questionKey:'purpose',answerText:'purpose',structuredValue:null,resolution:'ANSWERED',explanation:null,revision:1,updatedAt:'2026-10-03T00:00:00Z'};
 const gate={paperId:requestId,phaseCode:'PRE',definitionVersion:1,complete:false,issues:[],phaseRevision:1,inputSnapshotHash:'a'.repeat(64),evaluatedAt:'2026-10-03T00:00:00Z'};
 const advance={phase:{...phase,state:'COMPLETED',revision:2,completedAt:'2026-10-03T00:00:00Z'},gate:{...gate,complete:true,phaseRevision:2},nextPhase:'P1',paperLifecycle:'ACTIVE'};
 const responses=[phase,[answer],preDefinition,{...answer},gate,advance,{activePhaseCode:'PRE',activePhaseRevision:2},phase].map(data=>({contractVersion:1,requestId,ok:true,data}));
 const invoke=vi.fn(async(_command:string,_args:Record<string,unknown>)=>responses.shift());const api=createTauriApis(invoke);
 const results=await Promise.all([
  api.workflow.getPhase({requestId,paperId:requestId,phaseCode:'PRE'}),
  api.workflow.getPhaseAnswers({requestId,paperId:requestId,phaseCode:'PRE'}),
  api.workflow.getPhaseDefinition({requestId,phaseCode:'PRE',version:1}),
  api.workflow.savePhaseAnswer({requestId,paperId:requestId,phaseCode:'PRE',questionKey:'purpose',expectedRevision:0,answerText:'purpose',structuredValue:null,resolution:'ANSWERED',explanation:null}),
  api.workflow.evaluateGate({requestId,paperId:requestId,phaseCode:'PRE'}),
  api.workflow.advancePhase({requestId,paperId:requestId,fromPhase:'PRE',expectedPhaseRevision:1}),
  api.workflow.goBackToPhase({requestId,paperId:requestId,targetPhase:'PRE',expectedActivePhaseRevision:2}),
  api.workflow.touchPhase({requestId,paperId:requestId,phaseCode:'PRE',expectedActivePhaseRevision:2}),
 ]);
 expect(results.every(result=>result.ok)).toBe(true);
 expect(invoke.mock.calls.map(call=>call[0])).toEqual(['workflow_get_phase','workflow_get_phase_answers','workflow_get_phase_definition','workflow_save_phase_answer','workflow_evaluate_gate','workflow_advance_phase','workflow_go_back_to_phase','workflow_touch_phase']);
 const set=await api.workflow.setP3Candidate({requestId,paperId:requestId,itemId:requestId,selected:true,priority:1,rationale:'candidate',noCandidatesJustification:null,expectedWorkflowRevision:1});
 const summary=await api.workflow.getP3CandidateSummary({requestId,paperId:requestId});
 expect(set.ok).toBe(false);expect(summary.ok).toBe(false);if(!set.ok)expect(set.error.code).toBe('UnsupportedCapability');if(!summary.ok)expect(summary.error.code).toBe('UnsupportedCapability');expect(invoke).toHaveBeenCalledTimes(8);
});
it('invalid or extra outbound payload is rejected',async()=>{const invoke=vi.fn();const api=createTauriApis(invoke);const result=await api.settings.getAppInfo({requestId,extra:1} as {requestId:string});expect(result.ok).toBe(false);if(!result.ok)expect(result.error.code).toBe('InvalidInput');expect(invoke).not.toHaveBeenCalled();});
it('wrong request response is never accepted',async()=>{const api=createTauriApis(async()=>({...fixture,requestId:'00000000-0000-4000-8000-000000000002'}));const result=await api.settings.getAppInfo({requestId});expect(result.ok).toBe(false);if(!result.ok)expect(result.error.code).toBe('IntegrityFailure');});
it('invoke exceptions are normalized safely',async()=>{const api=createTauriApis(async()=>{throw 'C:/private/scientific source.pdf';});const result=await api.settings.getAppInfo({requestId});expect(JSON.stringify(result)).not.toContain('private');expect(result.ok).toBe(false);});
it('library and reader clients route every enabled command through the validated invoke adapter',async()=>{
 const opened={paper:{title:'Paper',authors:[],year:null,doi:null,venue:null,reviewType:'unknown',domain:null,id:requestId,documentId:requestId,lifecycle:'NEW',archivedFromLifecycle:null,activePhaseCode:null,processingInitialized:false,revision:0,createdAt:'2026-10-01T00:00:00Z',updatedAt:'2026-10-01T00:00:00Z',lastOpenedAt:null},document:{id:requestId,paperId:requestId,originalFilename:'synthetic.pdf',sha256:'a'.repeat(64),importedAt:'2026-10-01T00:00:00Z',status:'ACTIVE'},readingPosition:{documentId:requestId,pageIndex:1,zoom:1,revision:0,updatedAt:'2026-10-01T00:00:00Z'},documentUrl:`http://research.localhost/${requestId}`};
 const position={documentId:requestId,pageIndex:2,zoom:1.25,revision:1,updatedAt:'2026-10-01T00:00:00Z'};
 const responses=[{contractVersion:1,requestId,ok:true,data:opened},{contractVersion:1,requestId,ok:true,data:null},{contractVersion:1,requestId,ok:true,data:position},{contractVersion:1,requestId,ok:true,data:position}];const invoke=vi.fn(async(_command:string,_args:Record<string,unknown>)=>responses.shift());const api=createTauriApis(invoke);
 expect((await api.reader.openPaper({requestId,paperId:requestId})).ok).toBe(true);expect((await api.reader.getLastOpenedPaper({requestId})).ok).toBe(true);expect((await api.reader.getReadingPosition({requestId,documentId:requestId})).ok).toBe(true);expect((await api.reader.saveReadingPosition({requestId,documentId:requestId,expectedRevision:0,pageIndex:2,zoom:1.25})).ok).toBe(true);
 expect(invoke.mock.calls.map(call=>call[0])).toEqual(['reader_open_paper','reader_get_last_opened_paper','reader_get_reading_position','reader_save_reading_position']);
});

it('reader client accepts and preserves the previous confirmed UTC receipt and position timestamps',async()=>{
 const legacyTime='2026-10-03T14:36:22.575865800+00:00';
 const opened={paper:{title:'Paper sintético',authors:[],year:null,doi:null,venue:null,reviewType:'unknown',domain:null,id:requestId,documentId:requestId,lifecycle:'NEW',archivedFromLifecycle:null,activePhaseCode:null,processingInitialized:false,revision:0,createdAt:'2026-10-01T00:00:00Z',updatedAt:'2026-10-01T00:00:00Z',lastOpenedAt:legacyTime},document:{id:requestId,paperId:requestId,originalFilename:'synthetic.pdf',sha256:'a'.repeat(64),importedAt:'2026-10-01T00:00:00Z',status:'ACTIVE'},readingPosition:{documentId:requestId,pageIndex:1,zoom:1,revision:0,updatedAt:legacyTime},documentUrl:`http://research.localhost/${requestId}`};
 const paper=opened.paper;
 const position={documentId:requestId,pageIndex:2,zoom:1.25,revision:1,updatedAt:legacyTime};
 const responses=[opened,paper,position,position].map(data=>({contractVersion:1,requestId,ok:true,data}));
 const api=createTauriApis(async()=>responses.shift());
 const open=await api.reader.openPaper({requestId,paperId:requestId});
 const last=await api.reader.getLastOpenedPaper({requestId});
 const get=await api.reader.getReadingPosition({requestId,documentId:requestId});
 const save=await api.reader.saveReadingPosition({requestId,documentId:requestId,expectedRevision:0,pageIndex:2,zoom:1.25});
 expect(open).toMatchObject({ok:true,data:{paper:{lastOpenedAt:legacyTime},readingPosition:{updatedAt:legacyTime}}});
 expect(last).toMatchObject({ok:true,data:{lastOpenedAt:legacyTime}});
 expect(get).toMatchObject({ok:true,data:{updatedAt:legacyTime}});
 expect(save).toMatchObject({ok:true,data:{updatedAt:legacyTime}});
});
