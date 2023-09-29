dy --region local admin create table database --keys __pk __sk
sleep 1
dy -r local admin create index gsi1 -t database --keys __gsi1pk __gsi1sk
sleep 1
dy -r local admin create index gsi2 -t database --keys __gsi2pk __gsi2sk
sleep 1
dy -r local admin create index gsi3 -t database --keys __gsi3pk __gsi3sk
