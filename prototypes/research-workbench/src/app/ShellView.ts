export type ShellView = {kind:'Home'} | {kind:'Library'} | {kind:'PaperWorkspace';paperId:string} | {kind:'Knowledge';conceptId:string|null} | {kind:'Settings'};
