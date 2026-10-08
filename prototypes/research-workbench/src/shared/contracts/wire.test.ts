import {describe,it,expect} from 'vitest';
import {appInfoSchema,ipcSchema,paperMetadataSchema} from './wire';
import {backupDtoSchema,importPreviewDtoSchema,pageSchema,workflowGetPhaseDefinitionArgsSchema} from './wire';
import {paperDtoSchema,readingPositionDtoSchema} from './wire';
import backup from '../../../contracts/fixtures/backup-aggregate.json';
import preview from '../../../contracts/fixtures/import-preview.json';
import {z} from 'zod';
it('backup aggregate exceeds the individual PDF cap while PDF cannot',()=>{
 expect(backupDtoSchema.parse(backup)).toEqual(backup);
 expect(importPreviewDtoSchema.safeParse({...preview,sizeBytes:524288001}).success).toBe(false);
});
it('optional numbers accept omission and number but never explicit null',()=>{
 const page=pageSchema(z.string());
 expect(page.safeParse({items:[],nextCursor:null}).success).toBe(true);
 expect(page.safeParse({items:[],nextCursor:null,total:1}).success).toBe(true);
 expect(page.safeParse({items:[],nextCursor:null,total:null}).success).toBe(false);
 const args={requestId:'00000000-0000-4000-8000-000000000001',phaseCode:'PRE'};
 expect(workflowGetPhaseDefinitionArgsSchema.safeParse(args).success).toBe(true);
 expect(workflowGetPhaseDefinitionArgsSchema.safeParse({...args,version:1}).success).toBe(true);
 expect(workflowGetPhaseDefinitionArgsSchema.safeParse({...args,version:null}).success).toBe(false);
});
describe('wire contracts',()=>{
 it('ipc_envelope_camel_case_roundtrip',()=>{const input={contractVersion:1,requestId:'00000000-0000-4000-8000-000000000001',ok:true,data:{appVersion:'0.0.1',contractVersion:1,schemaVersion:1,libraryId:'00000000-0000-4000-8000-000000000002',libraryRootLabel:'Biblioteca local — investigación',state:'ready',capabilities:{settings:true,library:false}}};expect(ipcSchema(appInfoSchema).parse(input)).toEqual(input);});
 it('unknown_enum_and_payload_rejected',()=>{expect(()=>appInfoSchema.parse({state:'pretend'})).toThrow();expect(()=>paperMetadataSchema.parse({title:'Título',authors:[],year:2026,doi:null,venue:null,reviewType:'invented',domain:null})).toThrow();expect(()=>paperMetadataSchema.parse({title:'Título',authors:[],year:Infinity,doi:null,venue:null,reviewType:'survey',domain:null,extra:1})).toThrow();});
});

it('title limits count Unicode scalar characters consistently with Rust',()=>{const metadata={title:'🧪'.repeat(1000),authors:['Ana'],year:null,doi:null,venue:null,reviewType:'survey',domain:null};expect(paperMetadataSchema.parse(metadata).title).toBe(metadata.title);expect(()=>paperMetadataSchema.parse({...metadata,title:metadata.title+'🧪'})).toThrow();expect(()=>paperMetadataSchema.parse({...metadata,authors:['   ']})).toThrow();});

it('reader UTC timestamps accept only Z or explicit zero offset and preserve the wire string',()=>{
 const paper={title:'Paper sintético',authors:[],year:null,doi:null,venue:null,reviewType:'unknown',domain:null,id:'00000000-0000-4000-8000-000000000001',documentId:'00000000-0000-4000-8000-000000000002',lifecycle:'NEW',archivedFromLifecycle:null,activePhaseCode:null,processingInitialized:false,revision:0,createdAt:'2026-10-01T00:00:00Z',updatedAt:'2026-10-01T00:00:00Z',lastOpenedAt:null};
 const position={documentId:paper.documentId,pageIndex:1,zoom:1,revision:0,updatedAt:'2026-10-01T00:00:00Z'};
 const accepted=['2026-10-03T14:36:22.575865800Z','2026-10-03T14:36:22.575865800+00:00'];
 for(const timestamp of accepted){
  expect(paperDtoSchema.parse({...paper,lastOpenedAt:timestamp}).lastOpenedAt).toBe(timestamp);
  expect(readingPositionDtoSchema.parse({...position,updatedAt:timestamp}).updatedAt).toBe(timestamp);
 }
 for(const timestamp of ['2026-10-03T14:36:22.575865800-00:00','2026-10-03T14:36:22.575865800+01:00','2026-10-03T14:36:22.575865800-05:00','2026-10-03T14:36:22.575865800z','2026-10-03T14:36:22','2026-02-30T14:36:22Z','2026-10-03T25:36:22Z',42]){
  expect(paperDtoSchema.safeParse({...paper,lastOpenedAt:timestamp}).success).toBe(false);
  expect(readingPositionDtoSchema.safeParse({...position,updatedAt:timestamp}).success).toBe(false);
 }
});
