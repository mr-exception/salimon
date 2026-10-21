from pathlib import Path
from native_matrix import matrix
r=Path('artifacts/native-20261020');b=r/'SalimonIsolated.app/Contents/MacOS/salimon-client'
cases=[('empty-stable',b,[0],['onboard'],['settled'],30,2,3,100,1200),('settled',b,[500],['onboard'],['settled'],30,2,3,100,1200),('active',b,[500],['onboard'],['active'],3,0,10,90,0),('empty-launch',b,[0],['onboard'],['active'],3,0,10,90,0),('stress',b,[500],['scattered'],['settled'],5,2,1,120,120),('historical-before',Path('artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client'),[500],['onboard'],['settled'],30,2,3,100,1200),('historical-after',Path('artifacts/Salimon255Candidate.app/Contents/MacOS/salimon-client'),[500],['onboard'],['settled'],30,2,3,100,1200)]
for label,*args in cases:matrix('255-current/'+label,*args)
print('ALL CASES COMPLETE',flush=True)
