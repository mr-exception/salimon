import CoreGraphics
import CoreVideo
import Foundation
var count: UInt32 = 0
CGGetActiveDisplayList(0,nil,&count)
var ids=[CGDirectDisplayID](repeating:0,count:Int(count))
CGGetActiveDisplayList(count,&ids,&count)
var rows=[[String:Any]]()
for id in ids {
 var link: CVDisplayLink?
 let status=CVDisplayLinkCreateWithCGDisplay(id,&link)
 var row:[String:Any]=["display_id":id,"builtin":CGDisplayIsBuiltin(id) != 0,"main":CGDisplayIsMain(id) != 0,"bounds":NSStringFromRect(CGDisplayBounds(id)),"mode_refresh_hz":CGDisplayCopyDisplayMode(id)?.refreshRate ?? 0]
 if status == kCVReturnSuccess, let link=link {
  let time=CVDisplayLinkGetNominalOutputVideoRefreshPeriod(link)
  if time.timeValue>0 {row["nominal_refresh_hz"]=Double(time.timeScale)/Double(time.timeValue)}
 }
 rows.append(row)
}
let data=try JSONSerialization.data(withJSONObject:rows,options:[.prettyPrinted,.sortedKeys])
print(String(decoding:data,as:UTF8.self))
