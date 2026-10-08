import {invoke} from '@tauri-apps/api/core';
import {z} from 'zod';
import type * as Ports from '../../contracts/ports';
import type {IpcResult,IpcErrorCode,UUID} from '../../contracts/generated/contracts';
import * as Wire from '../../contracts/wire';
import manifest from '../../../../contracts/manifest.json';
export type Invoke = (command:string,args:Record<string,unknown>)=>Promise<unknown>;
const unsupported=(requestId:UUID):IpcResult<never>=>({contractVersion:1,requestId,ok:false,error:{code:'UnsupportedCapability',message:'Esta función todavía no está disponible.',retryable:false}});
function safeFailure(requestId:UUID,code:IpcErrorCode):IpcResult<never>{return {contractVersion:1,requestId,ok:false,error:{code,message:code==='InvalidInput'?'La solicitud contiene datos no válidos.':'No se pudo completar la operación. Revisa la biblioteca.',retryable:code==='StorageUnavailable'}};}
export function createTauriApis(invokeCommand:Invoke=invoke){
 async function call<T>(command:string,args:unknown,input:z.ZodType,payload:z.ZodType<T>):Promise<IpcResult<T>>{
  const parsed=input.safeParse(args);const id=Wire.uuidSchema.safeParse((args as {requestId?:unknown})?.requestId);const requestId=id.success?id.data:'00000000-0000-0000-0000-000000000000';
  if(!parsed.success)return safeFailure(requestId,'InvalidInput');
  const record=manifest.commands.find(entry=>entry.name===command);if(!record?.implemented)return unsupported(requestId);
  let response:unknown;
  try{response=await invokeCommand(command,{args:parsed.data});}
  catch(error){const envelope=Wire.ipcSchema(payload).safeParse(error);if(envelope.success&&envelope.data.requestId===requestId)return envelope.data;return safeFailure(requestId,'StorageUnavailable');}
  const result=Wire.ipcSchema(payload).safeParse(response);
  if(!result.success||result.data.requestId!==requestId)return safeFailure(requestId,'IntegrityFailure');
  return result.data;
 }
 const library:Ports.LibraryApi={
selectPdf:(requestId)=>call("library_select_pdf",{requestId},Wire.librarySelectPdfArgsSchema,Wire.importPreviewDtoSchema.nullable()),
confirmImport:(args)=>call("library_confirm_import",args,Wire.libraryConfirmImportArgsSchema,Wire.paperDtoSchema),
cancelImport:(args)=>call("library_cancel_import",args,Wire.libraryCancelImportArgsSchema,z.null()),
listPapers:(args)=>call("library_list_papers",args,Wire.libraryListPapersArgsSchema,Wire.pageSchema(Wire.paperDtoSchema)),
getPaper:(args)=>call("library_get_paper",args,Wire.libraryGetPaperArgsSchema,Wire.paperDtoSchema),
updateMetadata:(args)=>call("library_update_metadata",args,Wire.libraryUpdateMetadataArgsSchema,Wire.paperDtoSchema),
archivePaper:(args)=>call("library_archive_paper",args,Wire.libraryArchivePaperArgsSchema,Wire.paperDtoSchema),
restorePaper:(args)=>call("library_restore_paper",args,Wire.libraryRestorePaperArgsSchema,Wire.paperDtoSchema),
 };
 const reader:Ports.ReaderApi={
openPaper:(args)=>call("reader_open_paper",args,Wire.readerOpenPaperArgsSchema,Wire.openPaperDtoSchema),
getLastOpenedPaper:(args)=>call("reader_get_last_opened_paper",args,Wire.readerGetLastOpenedPaperArgsSchema,Wire.paperDtoSchema.nullable()),
getReadingPosition:(args)=>call("reader_get_reading_position",args,Wire.readerGetReadingPositionArgsSchema,Wire.readingPositionDtoSchema),
saveReadingPosition:(args)=>call("reader_save_reading_position",args,Wire.readerSaveReadingPositionArgsSchema,Wire.readingPositionDtoSchema),
 };
 const workflow:Ports.WorkflowApi={
getPhase:(args)=>call("workflow_get_phase",args,Wire.workflowGetPhaseArgsSchema,Wire.phaseDtoSchema),
getPhaseAnswers:(args)=>call("workflow_get_phase_answers",args,Wire.workflowGetPhaseAnswersArgsSchema,z.array(Wire.phaseAnswerDtoSchema)),
getPhaseDefinition:(args)=>call("workflow_get_phase_definition",args,Wire.workflowGetPhaseDefinitionArgsSchema,Wire.phaseDefinitionDtoSchema),
savePhaseAnswer:(args)=>call("workflow_save_phase_answer",args,Wire.workflowSavePhaseAnswerArgsSchema,Wire.phaseAnswerDtoSchema),
evaluateGate:(args)=>call("workflow_evaluate_gate",args,Wire.workflowEvaluateGateArgsSchema,Wire.gateEvaluationDtoSchema),
advancePhase:(args)=>call("workflow_advance_phase",args,Wire.workflowAdvancePhaseArgsSchema,Wire.workflowAdvancePhaseOutputSchema),
goBackToPhase:(args)=>call("workflow_go_back_to_phase",args,Wire.workflowGoBackToPhaseArgsSchema,Wire.workflowGoBackToPhaseOutputSchema),
touchPhase:(args)=>call("workflow_touch_phase",args,Wire.workflowTouchPhaseArgsSchema,Wire.phaseDtoSchema),
setP3Candidate:(args)=>call("workflow_set_p3_candidate",args,Wire.workflowSetP3CandidateArgsSchema,Wire.p3CandidateSummaryDtoSchema),
getP3CandidateSummary:(args)=>call("workflow_get_p3_candidate_summary",args,Wire.workflowGetP3CandidateSummaryArgsSchema,Wire.p3CandidateSummaryDtoSchema),
 };
 const knowledge:Ports.KnowledgeApi={
createItem:(args)=>call("knowledge_create_item",args,Wire.knowledgeCreateItemArgsSchema,Wire.knowledgeItemDtoSchema),
updateItem:(args)=>call("knowledge_update_item",args,Wire.knowledgeUpdateItemArgsSchema,Wire.knowledgeItemDtoSchema),
archiveItem:(args)=>call("knowledge_archive_item",args,Wire.knowledgeArchiveItemArgsSchema,Wire.knowledgeItemDtoSchema),
restoreItem:(args)=>call("knowledge_restore_item",args,Wire.knowledgeRestoreItemArgsSchema,Wire.knowledgeItemDtoSchema),
getKnowledgeItem:(args)=>call("knowledge_get_item",args,Wire.knowledgeGetItemArgsSchema,Wire.knowledgeItemDtoSchema),
listKnowledgeItems:(args)=>call("knowledge_list_items",args,Wire.knowledgeListItemsArgsSchema,Wire.pageSchema(Wire.knowledgeItemDtoSchema)),
 };
 const concept:Ports.ConceptApi={
suggestConcepts:(args)=>call("concept_suggest",args,Wire.conceptSuggestArgsSchema,z.array(Wire.conceptDtoSchema)),
getConcept:(args)=>call("concept_get",args,Wire.conceptGetArgsSchema,Wire.conceptDtoSchema),
listConceptItems:(args)=>call("concept_list_items",args,Wire.conceptListItemsArgsSchema,Wire.pageSchema(Wire.knowledgeItemDtoSchema)),
createConcept:(args)=>call("concept_create",args,Wire.conceptCreateArgsSchema,Wire.conceptDtoSchema),
updateConcept:(args)=>call("concept_update",args,Wire.conceptUpdateArgsSchema,Wire.conceptDtoSchema),
linkConcept:(args)=>call("concept_link",args,Wire.conceptLinkArgsSchema,z.null()),
archiveConcept:(args)=>call("concept_archive",args,Wire.conceptArchiveArgsSchema,Wire.conceptDtoSchema),
restoreConcept:(args)=>call("concept_restore",args,Wire.conceptRestoreArgsSchema,Wire.conceptDtoSchema),
 };
 const relation:Ports.RelationApi={
createRelation:(args)=>call("relation_create",args,Wire.relationCreateArgsSchema,Wire.relationDtoSchema),
updateRelation:(args)=>call("relation_update",args,Wire.relationUpdateArgsSchema,Wire.relationDtoSchema),
listRelations:(args)=>call("relation_list",args,Wire.relationListArgsSchema,z.array(Wire.relationDtoSchema)),
archiveRelation:(args)=>call("relation_archive",args,Wire.relationArchiveArgsSchema,Wire.relationDtoSchema),
restoreRelation:(args)=>call("relation_restore",args,Wire.relationRestoreArgsSchema,Wire.relationDtoSchema),
 };
 const provenance:Ports.ProvenanceApi={
attachLocator:(args)=>call("provenance_attach_locator",args,Wire.provenanceAttachLocatorArgsSchema,Wire.provenanceDtoSchema),
updateLocator:(args)=>call("provenance_update_locator",args,Wire.provenanceUpdateLocatorArgsSchema,Wire.provenanceDtoSchema),
getProvenance:(args)=>call("provenance_get",args,Wire.provenanceGetArgsSchema,Wire.provenanceDtoSchema),
checkDocumentHash:(args)=>call("provenance_check_document_hash",args,Wire.provenanceCheckDocumentHashArgsSchema,Wire.provenanceCheckDocumentHashOutputSchema),
 };
 const search:Ports.SearchApi={
searchLibrary:(args)=>call("search_library",args,Wire.searchLibraryArgsSchema,Wire.pageSchema(Wire.searchHitDtoSchema)),
searchKnowledge:(args)=>call("search_knowledge",args,Wire.searchKnowledgeArgsSchema,Wire.pageSchema(Wire.searchHitDtoSchema)),
 };
 const portability:Ports.PortabilityApi={
chooseExportDestination:(args)=>call("export_choose_destination",args,Wire.exportChooseDestinationArgsSchema,Wire.exportDestinationDtoSchema.nullable()),
exportLibrary:(args)=>call("export_library",args,Wire.exportLibraryArgsSchema,Wire.longOperationDtoSchema),
exportPaper:(args)=>call("export_paper",args,Wire.exportPaperArgsSchema,Wire.longOperationDtoSchema),
chooseBackupDestination:(args)=>call("backup_choose_destination",args,Wire.backupChooseDestinationArgsSchema,Wire.backupDestinationDtoSchema.nullable()),
createBackup:(args)=>call("backup_create",args,Wire.backupCreateArgsSchema,Wire.longOperationDtoSchema),
selectBackup:(args)=>call("backup_select",args,Wire.backupSelectArgsSchema,Wire.backupSelectOutputSchema.nullable()),
chooseRestoreTarget:(args)=>call("backup_choose_restore_target",args,Wire.backupChooseRestoreTargetArgsSchema,Wire.restoreTargetDtoSchema.nullable()),
getOperationStatus:(args)=>call("operation_get_status",args,Wire.operationGetStatusArgsSchema,Wire.longOperationDtoSchema),
cancelOperation:(args)=>call("operation_cancel",args,Wire.operationCancelArgsSchema,Wire.operationCancelOutputSchema),
verifyBackup:(args)=>call("backup_verify",args,Wire.backupVerifyArgsSchema,Wire.backupVerifyOutputSchema),
restoreBackup:(args)=>call("backup_restore",args,Wire.backupRestoreArgsSchema,Wire.longOperationDtoSchema),
 };
 const settings:Ports.SettingsApi={
getAppInfo:(args)=>call("settings_get_app_info",args,Wire.settingsGetAppInfoArgsSchema,Wire.appInfoDtoSchema),
getLibraryInfo:(args)=>call("settings_get_library_info",args,Wire.settingsGetLibraryInfoArgsSchema,Wire.libraryInfoDtoSchema),
selectLibrary:(args)=>call("settings_select_library",args,Wire.settingsSelectLibraryArgsSchema,Wire.settingsSelectLibraryOutputSchema.nullable()),
switchLibrary:(args)=>call("settings_switch_library",args,Wire.settingsSwitchLibraryArgsSchema,Wire.libraryInfoDtoSchema),
getLibraryStatus:(args)=>call("settings_get_library_status",args,Wire.settingsGetLibraryStatusArgsSchema,Wire.settingsGetLibraryStatusOutputSchema),
 };
 return {library,reader,workflow,knowledge,concept,relation,provenance,search,portability,settings};
}
