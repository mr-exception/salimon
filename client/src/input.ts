import { idleControls, type Controls } from './simulation.ts'
export type Axis=keyof Controls
export const axisForCode=(code:string):Axis|undefined=>({KeyW:'up',KeyS:'down',KeyA:'left',KeyD:'right'} as Record<string,Axis>)[code]
// Track independent physical inputs, so releasing one pointer/key cannot release
// another input holding the same axis. All owners are discarded on focus loss.
export class NavigationInput {
 private readonly held=new Map<string,Axis>()
 press(source:string,axis:Axis) {this.held.set(source,axis)}
 release(source:string) {this.held.delete(source)}
 releasePrefix(prefix:string) {for(const source of this.held.keys())if(source.startsWith(prefix))this.held.delete(source)}
 clear() {this.held.clear()}
 get controls():Controls {const controls=idleControls();for(const axis of this.held.values())controls[axis]=true;return controls}
}
export function acceptsFlightShortcut(target:EventTarget|null):boolean {
 return !(target instanceof Element && target.closest('input,textarea,select,button,a,[contenteditable]:not([contenteditable="false"])'))
}
export class FrameTimer {
 private previous:number
 constructor(now:number) {this.previous=now}
 resume(now:number) {this.previous=now}
 sample(now:number,hidden:boolean):number {const seconds=hidden?0:Math.max(0,now-this.previous)/1000;this.previous=now;return seconds}
}
