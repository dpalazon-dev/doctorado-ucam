import {it,expect} from 'vitest';
import app from '../../../contracts/fixtures/app-info.json';
import metadata from '../../../contracts/fixtures/metadata.json';
import preview from '../../../contracts/fixtures/import-preview.json';
import {ipcSchema,appInfoDtoSchema,paperMetadataInputSchema,importPreviewDtoSchema} from './wire';
it('Rust wire fixtures validate Unicode, long names and PDF boundary',()=>{expect(ipcSchema(appInfoDtoSchema).parse(app)).toEqual(app);expect(paperMetadataInputSchema.parse(metadata)).toEqual(metadata);expect(importPreviewDtoSchema.parse(preview)).toEqual(preview);expect(()=>importPreviewDtoSchema.parse({...preview,sizeBytes:524288001})).toThrow();});
