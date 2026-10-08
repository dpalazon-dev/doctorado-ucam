using System;
using System.IO;
using System.Text;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

// Scratch-only x64 probe. All paths are beneath a newly created synthetic fixture.
public static class HandleProbe {
  [StructLayout(LayoutKind.Sequential)] struct UnicodeString { public ushort Length, MaximumLength; public IntPtr Buffer; }
  [StructLayout(LayoutKind.Sequential)] struct ObjectAttributes { public uint Length; public IntPtr RootDirectory, ObjectName; public uint Attributes; public IntPtr SecurityDescriptor, SecurityQualityOfService; }
  [StructLayout(LayoutKind.Sequential)] struct IoStatus { public IntPtr Status; public UIntPtr Information; }
  [StructLayout(LayoutKind.Sequential)] struct RenameHeader { public byte Replace; public IntPtr Root; public uint Length; public ushort Name; }
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern SafeFileHandle CreateFileW(string path, uint access, uint share, IntPtr security, uint disposition, uint flags, IntPtr template);
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool DeviceIoControl(SafeFileHandle handle, uint code, byte[] input, uint size, IntPtr output, uint outputSize, out uint returned, IntPtr overlapped);
  [DllImport("ntdll.dll")] static extern int NtCreateFile(out IntPtr handle, uint access, ref ObjectAttributes attributes, out IoStatus status, IntPtr allocationSize, uint fileAttributes, uint share, uint disposition, uint options, IntPtr ea, uint eaLength);
  [DllImport("ntdll.dll")] static extern int NtSetInformationFile(SafeFileHandle handle, out IoStatus status, IntPtr info, uint length, int informationClass);
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern uint GetFinalPathNameByHandleW(SafeFileHandle handle, StringBuilder path, uint size, uint flags);
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool SetFileInformationByHandle(SafeFileHandle handle,int kind,IntPtr info,uint length);
  static SafeFileHandle Open(string path, uint access, uint share, bool dir) {
    var h=CreateFileW(path,access,share,IntPtr.Zero,3,0x00200000u|(dir?0x02000000u:0),IntPtr.Zero);
    if(h.IsInvalid) { int e=Marshal.GetLastWin32Error(); h.Dispose(); throw new IOException("CreateFile="+e); }
    return h;
  }
  static string Final(SafeFileHandle h) { var b=new StringBuilder(32768); uint n=GetFinalPathNameByHandleW(h,b,32768,0); if(n==0||n>=32768) throw new IOException("final-path failed"); return b.ToString(); }
  static SafeFileHandle Relative(SafeFileHandle root,string component,bool dir,uint disposition,out int nt,uint fileAccess=0,uint fileShare=1) {
    if(String.IsNullOrEmpty(component)||component=="."||component==".."||component.IndexOfAny(new[]{'\\','/',':','\0'})>=0) throw new ArgumentException("component");
    byte[] bytes=Encoding.Unicode.GetBytes(component); if(bytes.Length>65532) throw new ArgumentException("length");
    IntPtr name=Marshal.StringToHGlobalUni(component), us=Marshal.AllocHGlobal(Marshal.SizeOf<UnicodeString>());
    try {
      Marshal.StructureToPtr(new UnicodeString{Length=(ushort)bytes.Length,MaximumLength=(ushort)(bytes.Length+2),Buffer=name},us,false);
      var a=new ObjectAttributes{Length=(uint)Marshal.SizeOf<ObjectAttributes>(),RootDirectory=root.DangerousGetHandle(),ObjectName=us,Attributes=0x1040};
      IntPtr raw; IoStatus ios;
      uint access=dir?0x001000a1u:(fileAccess==0?0xc0110000u:fileAccess); // list/traverse/read-attrs/sync; default generic RW/delete/sync
      nt=NtCreateFile(out raw,access,ref a,out ios,IntPtr.Zero,0,dir?3u:fileShare,disposition,0x00200020u|(dir?1u:0x40u),IntPtr.Zero,0);
      Console.WriteLine("NtCreate component="+component+" status=0x"+nt.ToString("X8"));
      return nt>=0?new SafeFileHandle(raw,true):null;
    } finally { Marshal.FreeHGlobal(us); Marshal.FreeHGlobal(name); }
  }
  static int WinRename(SafeFileHandle source,string destination) {
    byte[] bytes=Encoding.Unicode.GetBytes(destination); int offset=(int)Marshal.OffsetOf<RenameHeader>("Name");
    int size=Marshal.SizeOf<RenameHeader>()+bytes.Length; IntPtr b=Marshal.AllocHGlobal(size);
    try {
      Marshal.Copy(new byte[size],0,b,size); Marshal.StructureToPtr(new RenameHeader{Replace=0,Root=IntPtr.Zero,Length=(uint)bytes.Length},b,false);
      Marshal.Copy(bytes,0,IntPtr.Add(b,offset),bytes.Length);
      bool ok=SetFileInformationByHandle(source,3,b,(uint)size); int e=ok?0:Marshal.GetLastWin32Error(); Console.WriteLine("WinRename absolute ok="+ok+" error="+e); return e;
    } finally { Marshal.FreeHGlobal(b); }
  }
  static int Rename(SafeFileHandle source,SafeFileHandle root,string name) {
    byte[] bytes=Encoding.Unicode.GetBytes(name); int offset=(int)Marshal.OffsetOf<RenameHeader>("Name");
    int size=Marshal.SizeOf<RenameHeader>()+bytes.Length; IntPtr b=Marshal.AllocHGlobal(size);
    try {
      Marshal.Copy(new byte[size],0,b,size); Marshal.StructureToPtr(new RenameHeader{Replace=0,Root=root.DangerousGetHandle(),Length=(uint)bytes.Length},b,false);
      Marshal.Copy(bytes,0,IntPtr.Add(b,offset),bytes.Length); IoStatus ios;
      int s=NtSetInformationFile(source,out ios,b,(uint)size,10); Console.WriteLine("NtRename name="+name+" status=0x"+s.ToString("X8")); return s;
    } finally { Marshal.FreeHGlobal(b); }
  }
  static bool Reparse(string path,string target,out int error) {
    using(var h=Open(path,0x100,7,true)) { // FILE_WRITE_ATTRIBUTES only, no WRITE_DATA
      byte[] sub=Encoding.Unicode.GetBytes(@"\??\"+target), print=Encoding.Unicode.GetBytes(target);
      byte[] buffer=new byte[16+sub.Length+2+print.Length+2];
      Array.Copy(BitConverter.GetBytes(0xa0000003u),0,buffer,0,4);
      Array.Copy(BitConverter.GetBytes((ushort)(buffer.Length-8)),0,buffer,4,2);
      Array.Copy(BitConverter.GetBytes((ushort)sub.Length),0,buffer,10,2);
      Array.Copy(BitConverter.GetBytes((ushort)(sub.Length+2)),0,buffer,12,2);
      Array.Copy(BitConverter.GetBytes((ushort)print.Length),0,buffer,14,2);
      Array.Copy(sub,0,buffer,16,sub.Length); Array.Copy(print,0,buffer,18+sub.Length,print.Length);
      uint returned; bool ok=DeviceIoControl(h,0x000900a4,buffer,(uint)buffer.Length,IntPtr.Zero,0,out returned,IntPtr.Zero);
      error=ok?0:Marshal.GetLastWin32Error(); Console.WriteLine("FSCTL path="+Path.GetFileName(path)+" WRITE_ATTRIBUTES-only ok="+ok+" error="+error); return ok;
    }
  }
  static void Require(bool ok,string check) { if(!ok) throw new Exception("ASSERT: "+check); Console.WriteLine("PASS "+check); }
  public static void Run(string fixture) {
    Require(IntPtr.Size==8,"x64 ABI"); Console.WriteLine("fixture="+fixture);
    Require(Marshal.SizeOf<ObjectAttributes>()==48&&Marshal.SizeOf<UnicodeString>()==16&&Marshal.SizeOf<IoStatus>()==16&&(int)Marshal.OffsetOf<RenameHeader>("Name")==20,"NT ABI sizes and rename tail offset");
    string outside=Path.Combine(fixture,"outside"); Directory.CreateDirectory(outside);
    string positive=Path.Combine(fixture,"positive"); Directory.CreateDirectory(positive);
    using(var d=Open(positive,0x001000a1,3,true)) {
      int nt; using(var f=Relative(d,"created-positive.bin",false,2,out nt)) {
        Require(nt==0&&f!=null,"positive relative create succeeds");
        Require(Final(f)==@"\\?\"+Path.Combine(positive,"created-positive.bin"),"positive relative create final path");
        using(var stream=new FileStream(f,FileAccess.ReadWrite)) { byte[] b=Encoding.UTF8.GetBytes("positive-created-bytes"); stream.Write(b,0,b.Length); stream.Flush(true); }
      }
      Require(File.ReadAllText(Path.Combine(positive,"created-positive.bin"))=="positive-created-bytes","positive create bytes");
      string from=Path.Combine(fixture,"positive-source.bin"); File.WriteAllText(from,"positive-renamed-bytes");
      using(var f=Open(from,0xc0110000,1,false)) {
        Require(Rename(f,d,"renamed-positive.bin")==0,"positive relative rename succeeds");
        Require(Final(f)==@"\\?\"+Path.Combine(positive,"renamed-positive.bin"),"positive relative rename final path");
      }
      Require(!File.Exists(from)&&File.ReadAllText(Path.Combine(positive,"renamed-positive.bin"))=="positive-renamed-bytes","positive rename bytes");
    }
    // Positive control makes privilege/buffer failures distinguishable from guards.
    string control=Path.Combine(fixture,"control"); Directory.CreateDirectory(control); int error;
    Require(Reparse(control,outside,out error),"unguarded empty directory accepts mount-point FSCTL with WRITE_ATTRIBUTES");
    File.WriteAllText(Path.Combine(outside,"marker"),"outside"); Require(File.ReadAllText(Path.Combine(control,"marker"))=="outside","control junction follows target");
    // Deny WRITE sharing alone does not close the attributes channel.
    string deny=Path.Combine(fixture,"denywrite"); Directory.CreateDirectory(deny);
    using(var guard=Open(deny,0x001000a1,1,true)) Require(Reparse(deny,outside,out error),"deny SHARE_WRITE still accepts attributes reparse");
    // Convert a retained NT-root directory in place, then create/rename through its handle.
    string dest=Path.Combine(fixture,"relative"); Directory.CreateDirectory(dest);
    using(var guard=Open(dest,0x001000a1,3,true)) {
      Require(Reparse(dest,outside,out error),"retained destination converted in place"); int nt;
      using(var child=Relative(guard,"created.bin",false,2,out nt)) {
        Require(nt<0||!Final(child).StartsWith(@"\\?\"+outside+"\\",StringComparison.OrdinalIgnoreCase),"relative create cannot escape");
        if(child!=null) Console.WriteLine("relative created final="+Final(child));
      }
      string source=Path.Combine(fixture,"source.bin"); File.WriteAllText(source,"source-bytes");
      using(var h=Open(source,0xc0110000,1,false)) {
        int s=Rename(h,guard,"promoted.bin");
        Require(s<0||!Final(h).StartsWith(@"\\?\"+outside+"\\",StringComparison.OrdinalIgnoreCase),"relative rename cannot escape");
        if(s>=0) Console.WriteLine("relative renamed final="+Final(h));
      }
      Require(!File.Exists(Path.Combine(outside,"created.bin"))&&!File.Exists(Path.Combine(outside,"promoted.bin")),"outside retains only marker");
    }
    // Existing target cannot be clobbered.
    string normal=Path.Combine(fixture,"normal"); Directory.CreateDirectory(normal); File.WriteAllText(Path.Combine(normal,"target.bin"),"racing-target");
    string staged=Path.Combine(fixture,"staged.bin"); File.WriteAllText(staged,"staged-original");
    using(var d=Open(normal,0x001000a1,3,true)) using(var h=Open(staged,0xc0110000,1,false)) Require(Rename(h,d,"target.bin")==unchecked((int)0xc0000035),"NT relative no-clobber STATUS_OBJECT_NAME_COLLISION");
    Require(File.ReadAllText(staged)=="staged-original"&&File.ReadAllText(Path.Combine(normal,"target.bin"))=="racing-target","collision preserves both bytes");
    // Minimal path-opening guard: nonempty child held without DELETE sharing.
    string pinned=Path.Combine(fixture,"pinned"); Directory.CreateDirectory(pinned); string pin=Path.Combine(pinned,"pin.bin"); File.WriteAllText(pin,"pin");
    using(var d=Open(pinned,0x001000a1,3,true)) using(var p=Open(pin,0x81,3,false)) {
      Require(!Reparse(pinned,outside,out error)&&error==145,"retained nonempty directory rejects FSCTL with ERROR_DIR_NOT_EMPTY");
      bool deleted=false; try {File.Delete(pin); deleted=true;} catch(IOException ex) { Console.WriteLine("pin File.Delete HResult=0x"+ex.HResult.ToString("X8")); Require(ex.HResult==unchecked((int)0x80070020),"pin deletion denied by sharing violation"); }
      Require(!deleted,"held child denies emptying parent");
    }
    File.Delete(pin); Require(!File.Exists(pin),"pin deletable after guard closes");
    // Candidate minimum: relative directory/pin acquisition, followed by Win32 absolute rename.
    string anchor=Path.Combine(fixture,"pin-relative-anchor"); Directory.CreateDirectory(anchor);
    using(var a=Open(anchor,0x001000a1,3,true)) {
      int nt; using(var d=Relative(a,"new-directory",true,2,out nt)) {
        Require(nt==0&&d!=null,"relative new directory acquisition succeeds");
        string dp=Path.Combine(anchor,"new-directory");
        using(var p=Relative(d,"reserved-pin.bin",false,2,out nt,0x00100081,3)) {
          Require(nt==0&&p!=null,"relative readonly pin create succeeds");
          Require(Final(d)==@"\\?\"+dp&&Final(p)==@"\\?\"+Path.Combine(dp,"reserved-pin.bin"),"relative pin binds same parent");
          Require(!Reparse(dp,outside,out error)&&error==145,"relative pin rejects attributes reparse");
          string from=Path.Combine(fixture,"win-source.bin"), target=Path.Combine(dp,"win-target.bin"); File.WriteAllText(from,"win-source-bytes");
          using(var f=Open(from,0xc0110000,1,false)) { Require(WinRename(f,target)==0,"absolute Win32 rename succeeds under pin"); Require(Final(f)==@"\\?\"+target,"absolute rename remains pinned parent"); }
          Require(File.ReadAllText(target)=="win-source-bytes","absolute rename bytes");
          string collision=Path.Combine(fixture,"win-collision.bin"); File.WriteAllText(collision,"collision-source");
          using(var f=Open(collision,0xc0110000,1,false)) Require(WinRename(f,target)==183,"absolute Win32 no-clobber ERROR_ALREADY_EXISTS");
          Require(File.ReadAllText(collision)=="collision-source"&&File.ReadAllText(target)=="win-source-bytes","absolute no-clobber preserves both bytes");
        }
        using(var p=Relative(d,"reserved-pin.bin",false,1,out nt,0x00100081,3)) Require(nt==0&&p!=null,"existing pin reopened without create/write access");
      }
    }
    string attrsOnly=Path.Combine(fixture,"attrs-only-pin.bin"); File.WriteAllText(attrsOnly,"attrs");
    using(var p=Open(attrsOnly,0x80,3,false)) {
      bool deleted=false; try { File.Delete(attrsOnly); deleted=true; } catch(IOException) {}
      Console.WriteLine("OBSERVATION attributes-only no-DELETE share allowed File.Delete="+deleted);
    }
    Console.WriteLine("ALL ASSERTIONS PASSED");
  }
}
