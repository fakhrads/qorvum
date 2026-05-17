(module
 (type $0 (func (param i32 i32) (result i32)))
 (type $1 (func (param i32 i32) (result i64)))
 (type $2 (func (param i32) (result i32)))
 (type $3 (func (param i32 i32 i32) (result i32)))
 (type $4 (func (param i32) (result i64)))
 (type $5 (func (param i32 i32 i32 i32)))
 (type $6 (func (param i32 i32 i32)))
 (type $7 (func (result i32)))
 (type $8 (func (param i32)))
 (type $9 (func (param i32 i32)))
 (type $10 (func (param i32 i32 i32 i32) (result i32)))
 (type $11 (func (param i64) (result i32)))
 (type $12 (func (param i32 i32 i32 i32) (result i64)))
 (type $13 (func))
 (import "env" "abort" (func $~lib/builtins/abort (param i32 i32 i32 i32)))
 (import "qv" "qv_insert" (func $~lib/qorvum-contract-sdk/assembly/index/__qv_insert (param i32 i32) (result i64)))
 (import "qv" "qv_emit_event" (func $~lib/qorvum-contract-sdk/assembly/index/__qv_emit_event (param i32 i32 i32 i32)))
 (import "qv" "qv_get" (func $~lib/qorvum-contract-sdk/assembly/index/__qv_get (param i32 i32) (result i64)))
 (import "qv" "qv_patch" (func $~lib/qorvum-contract-sdk/assembly/index/__qv_patch (param i32 i32) (result i64)))
 (import "qv" "qv_delete" (func $~lib/qorvum-contract-sdk/assembly/index/__qv_delete (param i32 i32) (result i64)))
 (import "qv" "qv_query" (func $~lib/qorvum-contract-sdk/assembly/index/__qv_query (param i32 i32) (result i64)))
 (import "qv" "qv_has_role" (func $~lib/qorvum-contract-sdk/assembly/index/__qv_has_role (param i32 i32) (result i32)))
 (global $~lib/rt/stub/offset (mut i32) (i32.const 0))
 (global $assembly/schema/UserSchema (mut i32) (i32.const 0))
 (global $assembly/schema/TodoSchema (mut i32) (i32.const 0))
 (global $~argumentsLength (mut i32) (i32.const 0))
 (memory $0 1)
 (data $0 (i32.const 1036) "<")
 (data $0.1 (i32.const 1048) "\02\00\00\00(\00\00\00A\00l\00l\00o\00c\00a\00t\00i\00o\00n\00 \00t\00o\00o\00 \00l\00a\00r\00g\00e")
 (data $1 (i32.const 1100) "<")
 (data $1.1 (i32.const 1112) "\02\00\00\00\1e\00\00\00~\00l\00i\00b\00/\00r\00t\00/\00s\00t\00u\00b\00.\00t\00s")
 (data $2 (i32.const 1164) ",")
 (data $2.1 (i32.const 1176) "\02\00\00\00\1c\00\00\00I\00n\00v\00a\00l\00i\00d\00 \00l\00e\00n\00g\00t\00h")
 (data $3 (i32.const 1212) ",")
 (data $3.1 (i32.const 1224) "\02\00\00\00\1a\00\00\00~\00l\00i\00b\00/\00a\00r\00r\00a\00y\00.\00t\00s")
 (data $4 (i32.const 1260) "\1c")
 (data $4.1 (i32.const 1272) "\02\00\00\00\n\00\00\00u\00s\00e\00r\00s")
 (data $5 (i32.const 1292) ",")
 (data $5.1 (i32.const 1304) "\02\00\00\00\0e\00\00\00d\00e\00f\00a\00u\00l\00t")
 (data $6 (i32.const 1340) "\1c")
 (data $6.1 (i32.const 1352) "\02\00\00\00\n\00\00\00t\00o\00d\00o\00s")
 (data $7 (i32.const 1372) "\1c")
 (data $7.1 (i32.const 1384) "\02\00\00\00\n\00\00\00t\00i\00t\00l\00e")
 (data $8 (i32.const 1404) "\1c")
 (data $8.1 (i32.const 1416) "\02\00\00\00\08\00\00\00T\00e\00x\00t")
 (data $9 (i32.const 1436) "\1c")
 (data $9.1 (i32.const 1448) "\02\00\00\00\0c\00\00\00s\00t\00a\00t\00u\00s")
 (data $10 (i32.const 1468) "\1c")
 (data $10.1 (i32.const 1480) "\02\00\00\00\08\00\00\00d\00o\00n\00e")
 (data $11 (i32.const 1500) "\1c")
 (data $11.1 (i32.const 1512) "\02\00\00\00\08\00\00\00B\00o\00o\00l")
 (data $12 (i32.const 1532) ",")
 (data $12.1 (i32.const 1544) "\02\00\00\00\16\00\00\00a\00s\00s\00i\00g\00n\00e\00e\00_\00i\00d")
 (data $13 (i32.const 1580) ",")
 (data $13.1 (i32.const 1592) "\02\00\00\00\10\00\00\00d\00u\00e\00_\00d\00a\00t\00e")
 (data $14 (i32.const 1628) ",")
 (data $14.1 (i32.const 1640) "\02\00\00\00\10\00\00\00a\00s\00s\00i\00g\00n\00e\00e")
 (data $15 (i32.const 1676) "<")
 (data $15.1 (i32.const 1688) "\02\00\00\00&\00\00\00~\00l\00i\00b\00/\00a\00r\00r\00a\00y\00b\00u\00f\00f\00e\00r\00.\00t\00s")
 (data $16 (i32.const 1740) ",")
 (data $16.1 (i32.const 1752) "\02\00\00\00\1c\00\00\00~\00l\00i\00b\00/\00s\00t\00r\00i\00n\00g\00.\00t\00s")
 (data $17 (i32.const 1788) "\1c")
 (data $17.1 (i32.const 1800) "\02")
 (data $18 (i32.const 1820) ",")
 (data $18.1 (i32.const 1832) "\02\00\00\00\16\00\00\00c\00r\00e\00a\00t\00e\00_\00t\00o\00d\00o")
 (data $19 (i32.const 1868) "\1c")
 (data $19.1 (i32.const 1880) "\02\00\00\00\04\00\00\00i\00d")
 (data $20 (i32.const 1900) "\1c")
 (data $20.1 (i32.const 1912) "\02\00\00\00\02\00\00\00\"")
 (data $21 (i32.const 1932) "\1c")
 (data $21.1 (i32.const 1944) "\02\00\00\00\06\00\00\00\"\00:\00\"")
 (data $22 (i32.const 1964) ",")
 (data $22.1 (i32.const 1976) "\02\00\00\00\1c\00\00\00i\00d\00 \00i\00s\00 \00r\00e\00q\00u\00i\00r\00e\00d")
 (data $23 (i32.const 2012) "<")
 (data $23.1 (i32.const 2024) "\02\00\00\00(\00\00\00{\00\"\00o\00k\00\"\00:\00f\00a\00l\00s\00e\00,\00\"\00e\00r\00r\00o\00r\00\"\00:")
 (data $24 (i32.const 2076) "\1c")
 (data $24.1 (i32.const 2088) "\02\00\00\00\04\00\00\00\\\00\"")
 (data $25 (i32.const 2108) "\1c")
 (data $25.1 (i32.const 2120) "\02\00\00\00\04\00\00\00\\\00\\")
 (data $26 (i32.const 2140) "\1c")
 (data $26.1 (i32.const 2152) "\02\00\00\00\04\00\00\00\\\00n")
 (data $27 (i32.const 2172) "\1c")
 (data $27.1 (i32.const 2184) "\02\00\00\00\04\00\00\00\\\00r")
 (data $28 (i32.const 2204) "\1c")
 (data $28.1 (i32.const 2216) "\02\00\00\00\04\00\00\00\\\00t")
 (data $29 (i32.const 2236) "\1c")
 (data $29.1 (i32.const 2248) "\02\00\00\00\02\00\00\00}")
 (data $30 (i32.const 2268) "<")
 (data $30.1 (i32.const 2280) "\02\00\00\00$\00\00\00U\00n\00p\00a\00i\00r\00e\00d\00 \00s\00u\00r\00r\00o\00g\00a\00t\00e")
 (data $31 (i32.const 2332) "<")
 (data $31.1 (i32.const 2344) "\02\00\00\00$\00\00\00I\00n\00d\00e\00x\00 \00o\00u\00t\00 \00o\00f\00 \00r\00a\00n\00g\00e")
 (data $32 (i32.const 2396) "<")
 (data $32.1 (i32.const 2408) "\02\00\00\00$\00\00\00~\00l\00i\00b\00/\00t\00y\00p\00e\00d\00a\00r\00r\00a\00y\00.\00t\00s")
 (data $33 (i32.const 2460) "<")
 (data $33.1 (i32.const 2472) "\02\00\00\00\"\00\00\00t\00i\00t\00l\00e\00 \00i\00s\00 \00r\00e\00q\00u\00i\00r\00e\00d")
 (data $34 (i32.const 2524) "<")
 (data $34.1 (i32.const 2536) "\02\00\00\00 \00\00\00{\00\"\00t\00\"\00:\00\"\00T\00e\00x\00t\00\"\00,\00\"\00v\00\"\00:")
 (data $35 (i32.const 2588) ",")
 (data $35.1 (i32.const 2600) "\02\00\00\00\0e\00\00\00P\00E\00N\00D\00I\00N\00G")
 (data $36 (i32.const 2636) "<")
 (data $36.1 (i32.const 2648) "\02\00\00\00 \00\00\00{\00\"\00t\00\"\00:\00\"\00B\00o\00o\00l\00\"\00,\00\"\00v\00\"\00:")
 (data $37 (i32.const 2700) "\1c")
 (data $37.1 (i32.const 2712) "\02\00\00\00\08\00\00\00t\00r\00u\00e")
 (data $38 (i32.const 2732) "\1c")
 (data $38.1 (i32.const 2744) "\02\00\00\00\n\00\00\00f\00a\00l\00s\00e")
 (data $39 (i32.const 2764) "|")
 (data $39.1 (i32.const 2776) "\02\00\00\00^\00\00\00E\00l\00e\00m\00e\00n\00t\00 \00t\00y\00p\00e\00 \00m\00u\00s\00t\00 \00b\00e\00 \00n\00u\00l\00l\00a\00b\00l\00e\00 \00i\00f\00 \00a\00r\00r\00a\00y\00 \00i\00s\00 \00h\00o\00l\00e\00y")
 (data $40 (i32.const 2892) "L")
 (data $40.1 (i32.const 2904) "\02\00\00\002\00\00\00m\00i\00s\00s\00i\00n\00g\00 \00r\00e\00q\00u\00i\00r\00e\00d\00 \00f\00i\00e\00l\00d\00s\00:\00 ")
 (data $41 (i32.const 2972) "\1c")
 (data $41.1 (i32.const 2984) "\02\00\00\00\04\00\00\00,\00 ")
 (data $42 (i32.const 3004) "\1c")
 (data $42.1 (i32.const 3016) "\0f\00\00\00\08\00\00\00\01")
 (data $43 (i32.const 3036) ",")
 (data $43.1 (i32.const 3048) "\02\00\00\00\1c\00\00\00{\00\"\00c\00o\00l\00l\00e\00c\00t\00i\00o\00n\00\"\00:")
 (data $44 (i32.const 3084) ",")
 (data $44.1 (i32.const 3096) "\02\00\00\00\1a\00\00\00,\00\"\00p\00a\00r\00t\00i\00t\00i\00o\00n\00\"\00:")
 (data $45 (i32.const 3132) "\1c")
 (data $45.1 (i32.const 3144) "\02\00\00\00\0c\00\00\00,\00\"\00i\00d\00\"\00:")
 (data $46 (i32.const 3164) ",")
 (data $46.1 (i32.const 3176) "\02\00\00\00\14\00\00\00,\00\"\00f\00i\00e\00l\00d\00s\00\"\00:")
 (data $47 (i32.const 3212) "\1c")
 (data $47.1 (i32.const 3224) "\02\00\00\00\02\00\00\00:")
 (data $48 (i32.const 3244) "\1c")
 (data $48.1 (i32.const 3256) "\02\00\00\00\02\00\00\00{")
 (data $49 (i32.const 3276) "\1c")
 (data $49.1 (i32.const 3288) "\02\00\00\00\02\00\00\00,")
 (data $50 (i32.const 3308) "L")
 (data $50.1 (i32.const 3320) "\02\00\00\00.\00\00\00n\00u\00l\00l\00 \00r\00e\00s\00p\00o\00n\00s\00e\00 \00f\00r\00o\00m\00 \00h\00o\00s\00t")
 (data $51 (i32.const 3388) ",")
 (data $51.1 (i32.const 3400) "\02\00\00\00\14\00\00\00{\00\"\00o\00k\00\"\00:\00t\00r\00u\00e")
 (data $52 (i32.const 3436) ",")
 (data $52.1 (i32.const 3448) "\02\00\00\00\0e\00\00\00\"\00d\00a\00t\00a\00\"\00:")
 (data $53 (i32.const 3484) "\1c")
 (data $53.1 (i32.const 3496) "\02\00\00\00\08\00\00\00n\00u\00l\00l")
 (data $54 (i32.const 3516) ",")
 (data $54.1 (i32.const 3528) "\02\00\00\00\10\00\00\00\"\00e\00r\00r\00o\00r\00\"\00:")
 (data $55 (i32.const 3564) "<")
 (data $55.1 (i32.const 3576) "\02\00\00\00$\00\00\00u\00n\00k\00n\00o\00w\00n\00 \00h\00o\00s\00t\00 \00e\00r\00r\00o\00r")
 (data $56 (i32.const 3628) ",")
 (data $56.1 (i32.const 3640) "\02\00\00\00\18\00\00\00T\00O\00D\00O\00_\00C\00R\00E\00A\00T\00E\00D")
 (data $57 (i32.const 3676) "\1c")
 (data $57.1 (i32.const 3688) "\02\00\00\00\0c\00\00\00{\00\"\00i\00d\00\"\00:")
 (data $58 (i32.const 3708) ",")
 (data $58.1 (i32.const 3720) "\02\00\00\00\12\00\00\00,\00\"\00t\00i\00t\00l\00e\00\"\00:")
 (data $59 (i32.const 3756) "<")
 (data $59.1 (i32.const 3768) "\02\00\00\00$\00\00\00{\00\"\00o\00k\00\"\00:\00t\00r\00u\00e\00,\00\"\00d\00a\00t\00a\00\"\00:")
 (data $60 (i32.const 3820) ",")
 (data $60.1 (i32.const 3832) "\02\00\00\00\10\00\00\00g\00e\00t\00_\00t\00o\00d\00o")
 (data $61 (i32.const 3868) "\1c")
 (data $61.1 (i32.const 3880) "\0f\00\00\00\08\00\00\00\02")
 (data $62 (i32.const 3900) "<")
 (data $62.1 (i32.const 3912) "\02\00\00\00 \00\00\00T\00o\00d\00o\00 \00n\00o\00t\00 \00f\00o\00u\00n\00d\00:\00 ")
 (data $63 (i32.const 3964) ",")
 (data $63.1 (i32.const 3976) "\02\00\00\00\10\00\00\00{\00\"\00t\00o\00d\00o\00\"\00:")
 (data $64 (i32.const 4012) ",")
 (data $64.1 (i32.const 4024) "\02\00\00\00\18\00\00\00,\00\"\00a\00s\00s\00i\00g\00n\00e\00e\00\"\00:")
 (data $65 (i32.const 4060) "<")
 (data $65.1 (i32.const 4072) "\02\00\00\00\"\00\00\00,\00\"\00a\00s\00s\00i\00g\00n\00e\00e\00\"\00:\00n\00u\00l\00l\00}")
 (data $66 (i32.const 4124) ",")
 (data $66.1 (i32.const 4136) "\02\00\00\00\1a\00\00\00c\00o\00m\00p\00l\00e\00t\00e\00_\00t\00o\00d\00o")
 (data $67 (i32.const 4172) "\1c")
 (data $67.1 (i32.const 4184) "\02\00\00\00\08\00\00\00D\00O\00N\00E")
 (data $68 (i32.const 4204) "\1c")
 (data $68.1 (i32.const 4216) "\0f\00\00\00\08\00\00\00\03")
 (data $69 (i32.const 4236) ",")
 (data $69.1 (i32.const 4248) "\02\00\00\00\1c\00\00\00T\00O\00D\00O\00_\00C\00O\00M\00P\00L\00E\00T\00E\00D")
 (data $70 (i32.const 4284) ",")
 (data $70.1 (i32.const 4296) "\02\00\00\00\16\00\00\00d\00e\00l\00e\00t\00e\00_\00t\00o\00d\00o")
 (data $71 (i32.const 4332) "\1c")
 (data $71.1 (i32.const 4344) "\02\00\00\00\0c\00\00\00r\00e\00a\00s\00o\00n")
 (data $72 (i32.const 4364) ",")
 (data $72.1 (i32.const 4376) "\02\00\00\00\14\00\00\00,\00\"\00r\00e\00a\00s\00o\00n\00\"\00:")
 (data $73 (i32.const 4412) "\1c")
 (data $73.1 (i32.const 4424) "\0f\00\00\00\08\00\00\00\04")
 (data $74 (i32.const 4444) ",")
 (data $74.1 (i32.const 4456) "\02\00\00\00\18\00\00\00T\00O\00D\00O\00_\00D\00E\00L\00E\00T\00E\00D")
 (data $75 (i32.const 4492) ",")
 (data $75.1 (i32.const 4504) "\02\00\00\00\14\00\00\00l\00i\00s\00t\00_\00t\00o\00d\00o\00s")
 (data $76 (i32.const 4540) "\1c")
 (data $76.1 (i32.const 4552) "\02\00\00\00\n\00\00\00l\00i\00m\00i\00t")
 (data $77 (i32.const 4572) "\1c")
 (data $77.1 (i32.const 4584) "\02\00\00\00\04\00\00\00\"\00:")
 (data $78 (i32.const 4604) ",")
 (data $78.1 (i32.const 4616) "\02\00\00\00\0e\00\00\00{\00\"\00E\00q\00\"\00:\00[")
 (data $79 (i32.const 4652) "<")
 (data $79.1 (i32.const 4664) "\02\00\00\00\"\00\00\00,\00{\00\"\00t\00\"\00:\00\"\00T\00e\00x\00t\00\"\00,\00\"\00v\00\"\00:")
 (data $80 (i32.const 4716) "\1c")
 (data $80.1 (i32.const 4728) "\02\00\00\00\06\00\00\00}\00]\00}")
 (data $81 (i32.const 4748) ",")
 (data $81.1 (i32.const 4760) "\02\00\00\00\12\00\00\00{\00\"\00f\00i\00e\00l\00d\00\"\00:")
 (data $82 (i32.const 4796) "<")
 (data $82.1 (i32.const 4808) "\02\00\00\00(\00\00\00,\00\"\00d\00e\00s\00c\00e\00n\00d\00i\00n\00g\00\"\00:\00f\00a\00l\00s\00e\00}")
 (data $83 (i32.const 4860) "\1c")
 (data $83.1 (i32.const 4872) "\02\00\00\00\02\00\00\00[")
 (data $84 (i32.const 4892) "\1c")
 (data $84.1 (i32.const 4904) "\02\00\00\00\02\00\00\00]")
 (data $85 (i32.const 4924) "\1c")
 (data $85.1 (i32.const 4936) "\0f\00\00\00\08\00\00\00\05")
 (data $86 (i32.const 4956) ",")
 (data $86.1 (i32.const 4968) "\02\00\00\00\14\00\00\00,\00\"\00f\00i\00l\00t\00e\00r\00\"\00:")
 (data $87 (i32.const 5004) ",")
 (data $87.1 (i32.const 5016) "\02\00\00\00\10\00\00\00,\00\"\00s\00o\00r\00t\00\"\00:")
 (data $88 (i32.const 5052) ",")
 (data $88.1 (i32.const 5064) "\02\00\00\00\12\00\00\00,\00\"\00l\00i\00m\00i\00t\00\"\00:")
 (data $89 (i32.const 5100) "|")
 (data $89.1 (i32.const 5112) "\02\00\00\00d\00\00\00t\00o\00S\00t\00r\00i\00n\00g\00(\00)\00 \00r\00a\00d\00i\00x\00 \00a\00r\00g\00u\00m\00e\00n\00t\00 \00m\00u\00s\00t\00 \00b\00e\00 \00b\00e\00t\00w\00e\00e\00n\00 \002\00 \00a\00n\00d\00 \003\006")
 (data $90 (i32.const 5228) "<")
 (data $90.1 (i32.const 5240) "\02\00\00\00&\00\00\00~\00l\00i\00b\00/\00u\00t\00i\00l\00/\00n\00u\00m\00b\00e\00r\00.\00t\00s")
 (data $91 (i32.const 5292) "\1c")
 (data $91.1 (i32.const 5304) "\02\00\00\00\02\00\00\000")
 (data $92 (i32.const 5324) "\\")
 (data $92.1 (i32.const 5336) "\02\00\00\00H\00\00\000\001\002\003\004\005\006\007\008\009\00a\00b\00c\00d\00e\00f\00g\00h\00i\00j\00k\00l\00m\00n\00o\00p\00q\00r\00s\00t\00u\00v\00w\00x\00y\00z")
 (data $93 (i32.const 5420) ",")
 (data $93.1 (i32.const 5432) "\02\00\00\00\16\00\00\00a\00s\00s\00i\00g\00n\00_\00t\00o\00d\00o")
 (data $94 (i32.const 5468) ",")
 (data $94.1 (i32.const 5480) "\02\00\00\00\0e\00\00\00M\00A\00N\00A\00G\00E\00R")
 (data $95 (i32.const 5516) "<")
 (data $95.1 (i32.const 5528) "\02\00\00\00*\00\00\00R\00e\00q\00u\00i\00r\00e\00s\00 \00M\00A\00N\00A\00G\00E\00R\00 \00r\00o\00l\00e")
 (data $96 (i32.const 5580) "L")
 (data $96.1 (i32.const 5592) "\02\00\00\00.\00\00\00a\00s\00s\00i\00g\00n\00e\00e\00_\00i\00d\00 \00i\00s\00 \00r\00e\00q\00u\00i\00r\00e\00d")
 (data $97 (i32.const 5660) "<")
 (data $97.1 (i32.const 5672) "\02\00\00\00 \00\00\00U\00s\00e\00r\00 \00n\00o\00t\00 \00f\00o\00u\00n\00d\00:\00 ")
 (data $98 (i32.const 5724) ",")
 (data $98.1 (i32.const 5736) "\02\00\00\00\1a\00\00\00T\00O\00D\00O\00_\00A\00S\00S\00I\00G\00N\00E\00D")
 (data $99 (i32.const 5772) "<")
 (data $99.1 (i32.const 5784) "\02\00\00\00\1e\00\00\00,\00\"\00a\00s\00s\00i\00g\00n\00e\00e\00_\00i\00d\00\"\00:")
 (data $100 (i32.const 5836) "<")
 (data $100.1 (i32.const 5848) "\02\00\00\00$\00\00\00U\00n\00k\00n\00o\00w\00n\00 \00f\00u\00n\00c\00t\00i\00o\00n\00:\00 ")
 (table $0 6 6 funcref)
 (elem $0 (i32.const 1) $~lib/qorvum-contract-sdk/assembly/index/__qv_insert $~lib/qorvum-contract-sdk/assembly/index/__qv_get $~lib/qorvum-contract-sdk/assembly/index/__qv_patch $~lib/qorvum-contract-sdk/assembly/index/__qv_delete $~lib/qorvum-contract-sdk/assembly/index/__qv_query)
 (export "dispatch" (func $assembly/index/dispatch))
 (export "alloc" (func $~lib/qorvum-contract-sdk/assembly/index/alloc))
 (export "memory" (memory $0))
 (start $~start)
 (func $~lib/rt/stub/maybeGrowMemory (param $0 i32)
  (local $1 i32)
  (local $2 i32)
  memory.size
  local.tee $1
  i32.const 16
  i32.shl
  i32.const 15
  i32.add
  i32.const -16
  i32.and
  local.tee $2
  local.get $0
  i32.lt_u
  if
   local.get $1
   local.get $0
   local.get $2
   i32.sub
   i32.const 65535
   i32.add
   i32.const -65536
   i32.and
   i32.const 16
   i32.shr_u
   local.tee $2
   local.get $1
   local.get $2
   i32.gt_s
   select
   memory.grow
   i32.const 0
   i32.lt_s
   if
    local.get $2
    memory.grow
    i32.const 0
    i32.lt_s
    if
     unreachable
    end
   end
  end
  local.get $0
  global.set $~lib/rt/stub/offset
 )
 (func $~lib/rt/stub/__alloc (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  local.get $0
  i32.const 1073741820
  i32.gt_u
  if
   i32.const 1056
   i32.const 1120
   i32.const 33
   i32.const 29
   call $~lib/builtins/abort
   unreachable
  end
  global.get $~lib/rt/stub/offset
  local.set $1
  global.get $~lib/rt/stub/offset
  i32.const 4
  i32.add
  local.tee $2
  local.get $0
  i32.const 19
  i32.add
  i32.const -16
  i32.and
  i32.const 4
  i32.sub
  local.tee $0
  i32.add
  call $~lib/rt/stub/maybeGrowMemory
  local.get $1
  local.get $0
  i32.store
  local.get $2
 )
 (func $~lib/rt/stub/__new (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  local.get $0
  i32.const 1073741804
  i32.gt_u
  if
   i32.const 1056
   i32.const 1120
   i32.const 86
   i32.const 30
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.const 16
  i32.add
  call $~lib/rt/stub/__alloc
  local.tee $3
  i32.const 4
  i32.sub
  local.tee $2
  i32.const 0
  i32.store offset=4
  local.get $2
  i32.const 0
  i32.store offset=8
  local.get $2
  local.get $1
  i32.store offset=12
  local.get $2
  local.get $0
  i32.store offset=16
  local.get $3
  i32.const 16
  i32.add
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/Schema#constructor (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  i32.const 16
  i32.const 4
  call $~lib/rt/stub/__new
  local.tee $1
  i32.const 0
  i32.store
  local.get $1
  i32.const 0
  i32.store offset=4
  i32.const 16
  i32.const 6
  call $~lib/rt/stub/__new
  local.tee $2
  i32.const 0
  i32.store
  local.get $2
  i32.const 0
  i32.store offset=4
  local.get $2
  i32.const 0
  i32.store offset=8
  local.get $2
  i32.const 0
  i32.store offset=12
  i32.const 32
  i32.const 1
  call $~lib/rt/stub/__new
  local.tee $3
  i32.const 0
  i32.const 32
  memory.fill
  local.get $2
  local.get $3
  i32.store
  local.get $2
  local.get $3
  i32.store offset=4
  local.get $2
  i32.const 32
  i32.store offset=8
  local.get $2
  i32.const 0
  i32.store offset=12
  local.get $1
  local.get $2
  i32.store offset=8
  i32.const 16
  i32.const 8
  call $~lib/rt/stub/__new
  local.tee $2
  i32.const 0
  i32.store
  local.get $2
  i32.const 0
  i32.store offset=4
  local.get $2
  i32.const 0
  i32.store offset=8
  local.get $2
  i32.const 0
  i32.store offset=12
  i32.const 32
  i32.const 1
  call $~lib/rt/stub/__new
  local.tee $3
  i32.const 0
  i32.const 32
  memory.fill
  local.get $2
  local.get $3
  i32.store
  local.get $2
  local.get $3
  i32.store offset=4
  local.get $2
  i32.const 32
  i32.store offset=8
  local.get $2
  i32.const 0
  i32.store offset=12
  local.get $1
  local.get $2
  i32.store offset=12
  local.get $1
  local.get $0
  i32.store
  local.get $1
  i32.const 1312
  i32.store offset=4
  local.get $1
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/Column#constructor (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  (local $3 i32)
  i32.const 9
  i32.const 5
  call $~lib/rt/stub/__new
  local.tee $3
  i32.const 0
  i32.store
  local.get $3
  i32.const 0
  i32.store offset=4
  local.get $3
  i32.const 0
  i32.store8 offset=8
  local.get $3
  local.get $0
  i32.store
  local.get $3
  local.get $1
  i32.store offset=4
  local.get $3
  local.get $2
  i32.store8 offset=8
  local.get $3
 )
 (func $~lib/rt/stub/__realloc (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  local.get $0
  i32.const 15
  i32.and
  i32.const 1
  local.get $0
  select
  if
   i32.const 0
   i32.const 1120
   i32.const 45
   i32.const 3
   call $~lib/builtins/abort
   unreachable
  end
  global.get $~lib/rt/stub/offset
  local.get $0
  i32.const 4
  i32.sub
  local.tee $4
  i32.load
  local.tee $3
  local.get $0
  i32.add
  i32.eq
  local.set $5
  local.get $1
  i32.const 19
  i32.add
  i32.const -16
  i32.and
  i32.const 4
  i32.sub
  local.set $2
  local.get $1
  local.get $3
  i32.gt_u
  if
   local.get $5
   if
    local.get $1
    i32.const 1073741820
    i32.gt_u
    if
     i32.const 1056
     i32.const 1120
     i32.const 52
     i32.const 33
     call $~lib/builtins/abort
     unreachable
    end
    local.get $0
    local.get $2
    i32.add
    call $~lib/rt/stub/maybeGrowMemory
    local.get $4
    local.get $2
    i32.store
   else
    local.get $2
    local.get $3
    i32.const 1
    i32.shl
    local.tee $1
    local.get $1
    local.get $2
    i32.lt_u
    select
    call $~lib/rt/stub/__alloc
    local.tee $1
    local.get $0
    local.get $3
    memory.copy
    local.get $1
    local.set $0
   end
  else
   local.get $5
   if
    local.get $0
    local.get $2
    i32.add
    global.set $~lib/rt/stub/offset
    local.get $4
    local.get $2
    i32.store
   end
  end
  local.get $0
 )
 (func $~lib/rt/stub/__renew (param $0 i32) (param $1 i32) (result i32)
  local.get $1
  i32.const 1073741804
  i32.gt_u
  if
   i32.const 1056
   i32.const 1120
   i32.const 99
   i32.const 30
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.const 16
  i32.sub
  local.get $1
  i32.const 16
  i32.add
  call $~lib/rt/stub/__realloc
  local.tee $0
  i32.const 4
  i32.sub
  local.get $1
  i32.store offset=16
  local.get $0
  i32.const 16
  i32.add
 )
 (func $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#push (param $0 i32) (param $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  local.get $0
  i32.load offset=12
  local.tee $3
  i32.const 1
  i32.add
  local.tee $4
  local.get $0
  i32.load offset=8
  local.tee $2
  i32.const 2
  i32.shr_u
  i32.gt_u
  if
   local.get $4
   i32.const 268435455
   i32.gt_u
   if
    i32.const 1184
    i32.const 1232
    i32.const 19
    i32.const 48
    call $~lib/builtins/abort
    unreachable
   end
   local.get $2
   local.get $0
   i32.load
   local.tee $5
   i32.const 1073741820
   local.get $2
   i32.const 1
   i32.shl
   local.tee $6
   local.get $6
   i32.const 1073741820
   i32.ge_u
   select
   local.tee $6
   i32.const 8
   local.get $4
   local.get $4
   i32.const 8
   i32.le_u
   select
   i32.const 2
   i32.shl
   local.tee $7
   local.get $6
   local.get $7
   i32.gt_u
   select
   local.tee $6
   call $~lib/rt/stub/__renew
   local.tee $7
   i32.add
   i32.const 0
   local.get $6
   local.get $2
   i32.sub
   memory.fill
   local.get $5
   local.get $7
   i32.ne
   if
    local.get $0
    local.get $7
    i32.store
    local.get $0
    local.get $7
    i32.store offset=4
   end
   local.get $0
   local.get $6
   i32.store offset=8
  end
  local.get $0
  i32.load offset=4
  local.get $3
  i32.const 2
  i32.shl
  i32.add
  local.get $1
  i32.store
  local.get $0
  local.get $4
  i32.store offset=12
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/Schema#text (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  local.get $0
  i32.load offset=8
  local.get $1
  i32.const 1424
  local.get $2
  call $~lib/qorvum-contract-sdk/assembly/index/Column#constructor
  call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#push
  local.get $0
 )
 (func $~lib/string/String.UTF8.decodeUnsafe (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  local.get $0
  local.get $1
  i32.add
  local.tee $3
  local.get $0
  i32.lt_u
  if
   i32.const 0
   i32.const 1760
   i32.const 770
   i32.const 7
   call $~lib/builtins/abort
   unreachable
  end
  local.get $1
  i32.const 1
  i32.shl
  i32.const 2
  call $~lib/rt/stub/__new
  local.tee $4
  local.set $1
  loop $while-continue|0
   local.get $0
   local.get $3
   i32.lt_u
   if
    block $while-break|0
     local.get $0
     i32.load8_u
     local.set $5
     local.get $0
     i32.const 1
     i32.add
     local.set $0
     local.get $5
     i32.const 128
     i32.and
     if
      local.get $0
      local.get $3
      i32.eq
      br_if $while-break|0
      local.get $0
      i32.load8_u
      i32.const 63
      i32.and
      local.set $6
      local.get $0
      i32.const 1
      i32.add
      local.set $0
      local.get $5
      i32.const 224
      i32.and
      i32.const 192
      i32.eq
      if
       local.get $1
       local.get $5
       i32.const 31
       i32.and
       i32.const 6
       i32.shl
       local.get $6
       i32.or
       i32.store16
      else
       local.get $0
       local.get $3
       i32.eq
       br_if $while-break|0
       local.get $0
       i32.load8_u
       i32.const 63
       i32.and
       local.set $2
       local.get $0
       i32.const 1
       i32.add
       local.set $0
       local.get $5
       i32.const 240
       i32.and
       i32.const 224
       i32.eq
       if
        local.get $5
        i32.const 15
        i32.and
        i32.const 12
        i32.shl
        local.get $6
        i32.const 6
        i32.shl
        i32.or
        local.get $2
        i32.or
        local.set $2
       else
        local.get $0
        local.get $3
        i32.eq
        br_if $while-break|0
        local.get $0
        i32.load8_u
        i32.const 63
        i32.and
        local.get $5
        i32.const 7
        i32.and
        i32.const 18
        i32.shl
        local.get $6
        i32.const 12
        i32.shl
        i32.or
        local.get $2
        i32.const 6
        i32.shl
        i32.or
        i32.or
        local.set $2
        local.get $0
        i32.const 1
        i32.add
        local.set $0
       end
       local.get $2
       i32.const 65536
       i32.lt_u
       if
        local.get $1
        local.get $2
        i32.store16
       else
        local.get $1
        local.get $2
        i32.const 65536
        i32.sub
        local.tee $2
        i32.const 10
        i32.shr_u
        i32.const 55296
        i32.or
        local.get $2
        i32.const 1023
        i32.and
        i32.const 56320
        i32.or
        i32.const 16
        i32.shl
        i32.or
        i32.store
        local.get $1
        i32.const 2
        i32.add
        local.set $1
       end
      end
     else
      local.get $1
      local.get $5
      i32.store16
     end
     local.get $1
     i32.const 2
     i32.add
     local.set $1
     br $while-continue|0
    end
   end
  end
  local.get $4
  local.get $1
  local.get $4
  i32.sub
  call $~lib/rt/stub/__renew
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/readString (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  i32.const 12
  i32.const 9
  call $~lib/rt/stub/__new
  local.tee $2
  i32.eqz
  if
   i32.const 12
   i32.const 3
   call $~lib/rt/stub/__new
   local.set $2
  end
  local.get $2
  i32.const 0
  i32.store
  local.get $2
  i32.const 0
  i32.store offset=4
  local.get $2
  i32.const 0
  i32.store offset=8
  local.get $1
  i32.const 1073741820
  i32.gt_u
  if
   i32.const 1184
   i32.const 1696
   i32.const 19
   i32.const 57
   call $~lib/builtins/abort
   unreachable
  end
  local.get $1
  i32.const 1
  call $~lib/rt/stub/__new
  local.tee $3
  i32.const 0
  local.get $1
  memory.fill
  local.get $2
  local.get $3
  i32.store
  local.get $2
  local.get $3
  i32.store offset=4
  local.get $2
  local.get $1
  i32.store offset=8
  local.get $2
  i32.load offset=4
  local.get $0
  local.get $1
  memory.copy
  local.get $2
  i32.load
  local.tee $0
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
  call $~lib/string/String.UTF8.decodeUnsafe
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/QvModel#constructor (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  i32.const 8
  i32.const 12
  call $~lib/rt/stub/__new
  local.tee $2
  i32.const 0
  i32.store
  local.get $2
  i32.const 0
  i32.store offset=4
  local.get $2
  local.get $0
  i32.store
  local.get $2
  local.get $1
  i32.store offset=4
  local.get $2
 )
 (func $~lib/util/string/compareImpl (param $0 i32) (param $1 i32) (param $2 i32) (param $3 i32) (result i32)
  (local $4 i32)
  local.get $0
  local.get $1
  i32.const 1
  i32.shl
  i32.add
  local.tee $1
  i32.const 7
  i32.and
  local.get $2
  i32.const 7
  i32.and
  i32.or
  i32.eqz
  local.get $3
  i32.const 4
  i32.ge_u
  i32.and
  if
   loop $do-loop|0
    local.get $1
    i64.load
    local.get $2
    i64.load
    i64.eq
    if
     local.get $1
     i32.const 8
     i32.add
     local.set $1
     local.get $2
     i32.const 8
     i32.add
     local.set $2
     local.get $3
     i32.const 4
     i32.sub
     local.tee $3
     i32.const 4
     i32.ge_u
     br_if $do-loop|0
    end
   end
  end
  loop $while-continue|1
   local.get $3
   local.tee $0
   i32.const 1
   i32.sub
   local.set $3
   local.get $0
   if
    local.get $1
    i32.load16_u
    local.tee $0
    local.get $2
    i32.load16_u
    local.tee $4
    i32.ne
    if
     local.get $0
     local.get $4
     i32.sub
     return
    end
    local.get $1
    i32.const 2
    i32.add
    local.set $1
    local.get $2
    i32.const 2
    i32.add
    local.set $2
    br $while-continue|1
   end
  end
  i32.const 0
 )
 (func $~lib/string/String.__eq (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  local.get $0
  local.get $1
  i32.eq
  if
   i32.const 1
   return
  end
  local.get $1
  i32.eqz
  local.get $0
  i32.eqz
  i32.or
  if
   i32.const 0
   return
  end
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  local.tee $2
  local.get $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  i32.ne
  if
   i32.const 0
   return
  end
  local.get $0
  i32.const 0
  local.get $1
  local.get $2
  call $~lib/util/string/compareImpl
  i32.eqz
 )
 (func $~lib/string/String.__concat (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  i32.const 1808
  local.set $2
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const -2
  i32.and
  local.tee $3
  local.get $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const -2
  i32.and
  local.tee $4
  i32.add
  local.tee $5
  if
   local.get $5
   i32.const 2
   call $~lib/rt/stub/__new
   local.tee $2
   local.get $0
   local.get $3
   memory.copy
   local.get $2
   local.get $3
   i32.add
   local.get $1
   local.get $4
   memory.copy
  end
  local.get $2
 )
 (func $~lib/string/String#indexOf (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  local.get $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  local.tee $3
  i32.eqz
  if
   i32.const 0
   return
  end
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  local.tee $4
  i32.eqz
  if
   i32.const -1
   return
  end
  local.get $4
  i32.const 0
  local.get $4
  i32.const 0
  i32.le_s
  select
  local.set $2
  local.get $4
  local.get $3
  i32.sub
  local.set $4
  loop $for-loop|0
   local.get $2
   local.get $4
   i32.le_s
   if
    local.get $0
    local.get $2
    local.get $1
    local.get $3
    call $~lib/util/string/compareImpl
    i32.eqz
    if
     local.get $2
     return
    end
    local.get $2
    i32.const 1
    i32.add
    local.set $2
    br $for-loop|0
   end
  end
  i32.const -1
 )
 (func $~lib/string/String#charCodeAt (param $0 i32) (param $1 i32) (result i32)
  local.get $1
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  i32.ge_u
  if
   i32.const -1
   return
  end
  local.get $0
  local.get $1
  i32.const 1
  i32.shl
  i32.add
  i32.load16_u
 )
 (func $~lib/string/String#substring (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  (local $3 i32)
  (local $4 i32)
  local.get $1
  i32.const 0
  local.get $1
  i32.const 0
  i32.gt_s
  select
  local.tee $3
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  local.tee $1
  local.get $1
  local.get $3
  i32.gt_s
  select
  local.tee $3
  local.get $2
  i32.const 0
  local.get $2
  i32.const 0
  i32.gt_s
  select
  local.tee $2
  local.get $1
  local.get $1
  local.get $2
  i32.gt_s
  select
  local.tee $2
  local.get $2
  local.get $3
  i32.gt_s
  select
  i32.const 1
  i32.shl
  local.set $4
  local.get $3
  local.get $2
  local.get $2
  local.get $3
  i32.lt_s
  select
  i32.const 1
  i32.shl
  local.tee $2
  local.get $4
  i32.sub
  local.tee $3
  i32.eqz
  if
   i32.const 1808
   return
  end
  local.get $4
  i32.eqz
  local.get $2
  local.get $1
  i32.const 1
  i32.shl
  i32.eq
  i32.and
  if
   local.get $0
   return
  end
  local.get $3
  i32.const 2
  call $~lib/rt/stub/__new
  local.tee $1
  local.get $0
  local.get $4
  i32.add
  local.get $3
  memory.copy
  local.get $1
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/getField (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  local.get $0
  i32.const 1920
  local.get $1
  call $~lib/string/String.__concat
  i32.const 1952
  call $~lib/string/String.__concat
  local.tee $1
  call $~lib/string/String#indexOf
  local.tee $2
  i32.const 0
  i32.lt_s
  if
   i32.const 1808
   return
  end
  local.get $2
  local.get $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  i32.add
  local.tee $2
  local.set $1
  loop $while-continue|0
   local.get $1
   local.get $0
   i32.const 20
   i32.sub
   i32.load offset=16
   i32.const 1
   i32.shr_u
   i32.lt_s
   if (result i32)
    local.get $0
    local.get $1
    call $~lib/string/String#charCodeAt
    i32.const 34
    i32.ne
   else
    i32.const 0
   end
   if
    local.get $1
    i32.const 1
    i32.add
    local.get $1
    local.get $0
    local.get $1
    call $~lib/string/String#charCodeAt
    i32.const 92
    i32.eq
    select
    i32.const 1
    i32.add
    local.set $1
    br $while-continue|0
   end
  end
  local.get $0
  local.get $2
  local.get $1
  call $~lib/string/String#substring
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/escStr (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  i32.const 1920
  local.set $1
  loop $for-loop|0
   local.get $3
   local.get $0
   i32.const 20
   i32.sub
   i32.load offset=16
   i32.const 1
   i32.shr_u
   i32.lt_s
   if
    local.get $0
    local.get $3
    call $~lib/string/String#charCodeAt
    local.tee $2
    i32.const 34
    i32.eq
    if (result i32)
     local.get $1
     i32.const 2096
     call $~lib/string/String.__concat
    else
     local.get $2
     i32.const 92
     i32.eq
     if (result i32)
      local.get $1
      i32.const 2128
      call $~lib/string/String.__concat
     else
      local.get $2
      i32.const 10
      i32.eq
      if (result i32)
       local.get $1
       i32.const 2160
       call $~lib/string/String.__concat
      else
       local.get $2
       i32.const 13
       i32.eq
       if (result i32)
        local.get $1
        i32.const 2192
        call $~lib/string/String.__concat
       else
        local.get $2
        i32.const 9
        i32.eq
        if (result i32)
         local.get $1
         i32.const 2224
         call $~lib/string/String.__concat
        else
         i32.const 1808
         local.set $2
         local.get $0
         i32.const 20
         i32.sub
         i32.load offset=16
         i32.const 1
         i32.shr_u
         local.get $3
         i32.gt_u
         if
          i32.const 2
          i32.const 2
          call $~lib/rt/stub/__new
          local.tee $2
          local.get $0
          local.get $3
          i32.const 1
          i32.shl
          i32.add
          i32.load16_u
          i32.store16
         end
         local.get $1
         local.get $2
         call $~lib/string/String.__concat
        end
       end
      end
     end
    end
    local.set $1
    local.get $3
    i32.const 1
    i32.add
    local.set $3
    br $for-loop|0
   end
  end
  local.get $1
  i32.const 1920
  call $~lib/string/String.__concat
 )
 (func $~lib/string/String.UTF8.encodeUnsafe (param $0 i32) (param $1 i32) (param $2 i32)
  (local $3 i32)
  (local $4 i32)
  local.get $0
  local.get $1
  i32.const 1
  i32.shl
  i32.add
  local.set $3
  local.get $2
  local.set $1
  loop $while-continue|0
   local.get $0
   local.get $3
   i32.lt_u
   if
    local.get $0
    i32.load16_u
    local.tee $2
    i32.const 128
    i32.lt_u
    if (result i32)
     local.get $1
     local.get $2
     i32.store8
     local.get $1
     i32.const 1
     i32.add
    else
     local.get $2
     i32.const 2048
     i32.lt_u
     if (result i32)
      local.get $1
      local.get $2
      i32.const 6
      i32.shr_u
      i32.const 192
      i32.or
      local.get $2
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.const 8
      i32.shl
      i32.or
      i32.store16
      local.get $1
      i32.const 2
      i32.add
     else
      local.get $2
      i32.const 56320
      i32.lt_u
      local.get $0
      i32.const 2
      i32.add
      local.get $3
      i32.lt_u
      i32.and
      local.get $2
      i32.const 63488
      i32.and
      i32.const 55296
      i32.eq
      i32.and
      if
       local.get $0
       i32.load16_u offset=2
       local.tee $4
       i32.const 64512
       i32.and
       i32.const 56320
       i32.eq
       if
        local.get $1
        local.get $2
        i32.const 1023
        i32.and
        i32.const 10
        i32.shl
        i32.const 65536
        i32.add
        local.get $4
        i32.const 1023
        i32.and
        i32.or
        local.tee $2
        i32.const 63
        i32.and
        i32.const 128
        i32.or
        i32.const 24
        i32.shl
        local.get $2
        i32.const 6
        i32.shr_u
        i32.const 63
        i32.and
        i32.const 128
        i32.or
        i32.const 16
        i32.shl
        i32.or
        local.get $2
        i32.const 12
        i32.shr_u
        i32.const 63
        i32.and
        i32.const 128
        i32.or
        i32.const 8
        i32.shl
        i32.or
        local.get $2
        i32.const 18
        i32.shr_u
        i32.const 240
        i32.or
        i32.or
        i32.store
        local.get $1
        i32.const 4
        i32.add
        local.set $1
        local.get $0
        i32.const 4
        i32.add
        local.set $0
        br $while-continue|0
       end
      end
      local.get $1
      local.get $2
      i32.const 12
      i32.shr_u
      i32.const 224
      i32.or
      local.get $2
      i32.const 6
      i32.shr_u
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.const 8
      i32.shl
      i32.or
      i32.store16
      local.get $1
      local.get $2
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.store8 offset=2
      local.get $1
      i32.const 3
      i32.add
     end
    end
    local.set $1
    local.get $0
    i32.const 2
    i32.add
    local.set $0
    br $while-continue|0
   end
  end
 )
 (func $~lib/string/String.UTF8.encode@varargs (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  block $2of2
   block $outOfRange
    global.get $~argumentsLength
    i32.const 1
    i32.sub
    br_table $2of2 $2of2 $2of2 $outOfRange
   end
   unreachable
  end
  local.get $0
  local.tee $1
  i32.const 20
  i32.sub
  i32.load offset=16
  local.get $1
  i32.add
  local.set $3
  loop $while-continue|0
   local.get $1
   local.get $3
   i32.lt_u
   if
    local.get $1
    i32.load16_u
    local.tee $4
    i32.const 128
    i32.lt_u
    if (result i32)
     local.get $2
     i32.const 1
     i32.add
    else
     local.get $4
     i32.const 2048
     i32.lt_u
     if (result i32)
      local.get $2
      i32.const 2
      i32.add
     else
      local.get $4
      i32.const 64512
      i32.and
      i32.const 55296
      i32.eq
      local.get $1
      i32.const 2
      i32.add
      local.get $3
      i32.lt_u
      i32.and
      if
       local.get $1
       i32.load16_u offset=2
       i32.const 64512
       i32.and
       i32.const 56320
       i32.eq
       if
        local.get $2
        i32.const 4
        i32.add
        local.set $2
        local.get $1
        i32.const 4
        i32.add
        local.set $1
        br $while-continue|0
       end
      end
      local.get $2
      i32.const 3
      i32.add
     end
    end
    local.set $2
    local.get $1
    i32.const 2
    i32.add
    local.set $1
    br $while-continue|0
   end
  end
  local.get $2
  i32.const 1
  call $~lib/rt/stub/__new
  local.set $1
  local.get $0
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  local.get $1
  call $~lib/string/String.UTF8.encodeUnsafe
  local.get $1
 )
 (func $~lib/typedarray/Uint8Array.wrap@varargs (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  block $2of2
   block $1of2
    block $outOfRange
     global.get $~argumentsLength
     i32.const 1
     i32.sub
     br_table $1of2 $1of2 $2of2 $outOfRange
    end
    unreachable
   end
   i32.const -1
   local.set $1
  end
  local.get $0
  local.tee $2
  i32.const 20
  i32.sub
  i32.load offset=16
  local.set $3
  local.get $1
  local.tee $0
  i32.const 0
  i32.lt_s
  if
   local.get $0
   i32.const -1
   i32.eq
   if (result i32)
    local.get $3
   else
    i32.const 1184
    i32.const 2416
    i32.const 1869
    i32.const 7
    call $~lib/builtins/abort
    unreachable
   end
   local.set $0
  else
   local.get $0
   local.get $3
   i32.gt_s
   if
    i32.const 1184
    i32.const 2416
    i32.const 1874
    i32.const 7
    call $~lib/builtins/abort
    unreachable
   end
  end
  i32.const 12
  i32.const 9
  call $~lib/rt/stub/__new
  local.tee $1
  local.get $2
  i32.store
  local.get $1
  local.get $0
  i32.store offset=8
  local.get $1
  local.get $2
  i32.store offset=4
  local.get $1
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/writeString (param $0 i32) (result i64)
  (local $1 i32)
  (local $2 i32)
  i32.const 1
  global.set $~argumentsLength
  local.get $0
  call $~lib/string/String.UTF8.encode@varargs
  local.set $0
  i32.const 1
  global.set $~argumentsLength
  local.get $0
  call $~lib/typedarray/Uint8Array.wrap@varargs
  local.tee $2
  i32.load offset=8
  local.tee $1
  call $~lib/rt/stub/__alloc
  local.tee $0
  local.get $2
  i32.load offset=4
  local.get $1
  memory.copy
  local.get $1
  i64.extend_i32_s
  local.get $0
  i64.extend_i32_s
  i64.const 32
  i64.shl
  i64.or
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/qv_err (param $0 i32) (result i64)
  i32.const 2032
  local.get $0
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/writeString
 )
 (func $~lib/array/Array<~lib/string/String>#constructor (result i32)
  (local $0 i32)
  (local $1 i32)
  i32.const 16
  i32.const 14
  call $~lib/rt/stub/__new
  local.tee $0
  i32.const 0
  i32.store
  local.get $0
  i32.const 0
  i32.store offset=4
  local.get $0
  i32.const 0
  i32.store offset=8
  local.get $0
  i32.const 0
  i32.store offset=12
  i32.const 32
  i32.const 1
  call $~lib/rt/stub/__new
  local.tee $1
  i32.const 0
  i32.const 32
  memory.fill
  local.get $0
  local.get $1
  i32.store
  local.get $0
  local.get $1
  i32.store offset=4
  local.get $0
  i32.const 32
  i32.store offset=8
  local.get $0
  i32.const 0
  i32.store offset=12
  local.get $0
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/Fields#constructor (result i32)
  (local $0 i32)
  i32.const 8
  i32.const 13
  call $~lib/rt/stub/__new
  local.tee $0
  i32.eqz
  if
   i32.const 0
   i32.const 0
   call $~lib/rt/stub/__new
   local.set $0
  end
  local.get $0
  call $~lib/array/Array<~lib/string/String>#constructor
  i32.store
  local.get $0
  call $~lib/array/Array<~lib/string/String>#constructor
  i32.store offset=4
  local.get $0
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/Fields#_set (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  local.get $0
  i32.load
  local.get $1
  call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#push
  local.get $0
  i32.load offset=4
  local.get $2
  call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#push
  local.get $0
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/Fields#text (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  local.get $0
  local.get $1
  i32.const 2544
  local.get $2
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/Fields#_set
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/Fields#bool (param $0 i32) (param $1 i32) (result i32)
  local.get $0
  i32.const 1488
  i32.const 2656
  i32.const 2720
  i32.const 2752
  local.get $1
  select
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/Fields#_set
 )
 (func $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#__get (param $0 i32) (param $1 i32) (result i32)
  local.get $1
  local.get $0
  i32.load offset=12
  i32.ge_u
  if
   i32.const 2352
   i32.const 1232
   i32.const 114
   i32.const 42
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.load offset=4
  local.get $1
  i32.const 2
  i32.shl
  i32.add
  i32.load
  local.tee $0
  i32.eqz
  if
   i32.const 2784
   i32.const 1232
   i32.const 118
   i32.const 40
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
 )
 (func $~lib/util/string/joinReferenceArray<~lib/string/String> (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  local.get $1
  i32.const 1
  i32.sub
  local.tee $3
  i32.const 0
  i32.lt_s
  if
   i32.const 1808
   return
  end
  local.get $3
  i32.eqz
  if
   i32.const 1808
   local.get $0
   i32.load
   local.tee $0
   local.get $0
   i32.const 0
   call $~lib/string/String.__eq
   select
   return
  end
  i32.const 1808
  local.set $1
  local.get $2
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  local.set $5
  loop $for-loop|0
   local.get $3
   local.get $4
   i32.gt_s
   if
    local.get $0
    local.get $4
    i32.const 2
    i32.shl
    i32.add
    i32.load
    local.tee $6
    i32.const 0
    call $~lib/string/String.__eq
    i32.eqz
    if
     local.get $1
     local.get $6
     call $~lib/string/String.__concat
     local.set $1
    end
    local.get $5
    if
     local.get $1
     local.get $2
     call $~lib/string/String.__concat
     local.set $1
    end
    local.get $4
    i32.const 1
    i32.add
    local.set $4
    br $for-loop|0
   end
  end
  local.get $0
  local.get $3
  i32.const 2
  i32.shl
  i32.add
  i32.load
  local.tee $0
  i32.const 0
  call $~lib/string/String.__eq
  if (result i32)
   local.get $1
  else
   local.get $1
   local.get $0
   call $~lib/string/String.__concat
  end
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/Fields#toJSON (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  call $~lib/array/Array<~lib/string/String>#constructor
  local.set $2
  loop $for-loop|0
   local.get $1
   local.get $0
   i32.load
   i32.load offset=12
   i32.lt_s
   if
    local.get $2
    local.get $0
    i32.load
    local.get $1
    call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#__get
    call $~lib/qorvum-contract-sdk/assembly/index/escStr
    i32.const 3232
    call $~lib/string/String.__concat
    local.get $0
    i32.load offset=4
    local.get $1
    call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#__get
    call $~lib/string/String.__concat
    call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#push
    local.get $1
    i32.const 1
    i32.add
    local.set $1
    br $for-loop|0
   end
  end
  i32.const 3264
  local.get $2
  i32.load offset=4
  local.get $2
  i32.load offset=12
  i32.const 3296
  call $~lib/util/string/joinReferenceArray<~lib/string/String>
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/parseEnvelope (param $0 i64) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  i32.const 12
  i32.const 16
  call $~lib/rt/stub/__new
  local.tee $1
  i32.eqz
  if
   i32.const 0
   i32.const 0
   call $~lib/rt/stub/__new
   local.set $1
  end
  local.get $1
  i32.const 0
  i32.store8
  local.get $1
  i32.const 1808
  i32.store offset=4
  local.get $1
  i32.const 1808
  i32.store offset=8
  local.get $0
  i64.const 32
  i64.shr_s
  i32.wrap_i64
  local.tee $2
  i32.eqz
  local.get $0
  i64.const 4294967295
  i64.and
  i32.wrap_i64
  local.tee $3
  i32.const 0
  i32.le_s
  i32.or
  if
   local.get $1
   i32.const 3328
   i32.store offset=8
   local.get $1
   return
  end
  local.get $2
  local.get $3
  call $~lib/qorvum-contract-sdk/assembly/index/readString
  local.tee $3
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  local.set $2
  local.get $2
  i32.const 3404
  i32.load
  i32.const 1
  i32.shr_u
  local.tee $4
  i32.lt_s
  if (result i32)
   i32.const 1
  else
   local.get $3
   i32.const 0
   i32.const 3408
   local.get $4
   call $~lib/util/string/compareImpl
  end
  if
   local.get $3
   i32.const 3536
   call $~lib/string/String#indexOf
   local.tee $2
   i32.const 0
   i32.ge_s
   if
    local.get $1
    local.get $3
    local.get $2
    i32.const 3532
    i32.load
    i32.const 1
    i32.shr_u
    i32.add
    local.get $3
    i32.const 20
    i32.sub
    i32.load offset=16
    i32.const 1
    i32.shr_u
    i32.const 1
    i32.sub
    call $~lib/string/String#substring
    local.tee $2
    i32.const 20
    i32.sub
    i32.load offset=16
    i32.const 1
    i32.shr_u
    i32.const 2
    i32.ge_u
    if (result i32)
     local.get $2
     i32.const 0
     call $~lib/string/String#charCodeAt
     i32.const 34
     i32.eq
    else
     i32.const 0
    end
    if (result i32)
     local.get $2
     i32.const 1
     local.get $2
     i32.const 20
     i32.sub
     i32.load offset=16
     i32.const 1
     i32.shr_u
     i32.const 1
     i32.sub
     call $~lib/string/String#substring
    else
     local.get $2
    end
    i32.store offset=8
   else
    local.get $1
    i32.const 3584
    i32.store offset=8
   end
  else
   local.get $1
   i32.const 1
   i32.store8
   local.get $1
   local.get $3
   i32.const 3456
   call $~lib/string/String#indexOf
   local.tee $2
   i32.const 0
   i32.ge_s
   if (result i32)
    local.get $3
    local.get $2
    i32.const 3452
    i32.load
    i32.const 1
    i32.shr_u
    i32.add
    local.get $3
    i32.const 20
    i32.sub
    i32.load offset=16
    i32.const 1
    i32.shr_u
    i32.const 1
    i32.sub
    call $~lib/string/String#substring
   else
    i32.const 3504
   end
   i32.store offset=4
  end
  local.get $1
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/Context#_exec (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  (local $3 i32)
  i32.const 1
  global.set $~argumentsLength
  local.get $2
  call $~lib/string/String.UTF8.encode@varargs
  local.set $2
  i32.const 1
  global.set $~argumentsLength
  local.get $2
  call $~lib/typedarray/Uint8Array.wrap@varargs
  local.tee $2
  i32.load offset=8
  call $~lib/rt/stub/__alloc
  local.tee $3
  local.get $2
  i32.load offset=4
  local.get $2
  i32.load offset=8
  memory.copy
  local.get $2
  i32.load offset=8
  local.set $2
  i32.const 2
  global.set $~argumentsLength
  local.get $3
  local.get $2
  local.get $1
  i32.load
  call_indirect (type $1)
  call $~lib/qorvum-contract-sdk/assembly/index/parseEnvelope
  local.tee $1
  i32.load8_u
  i32.eqz
  if
   local.get $0
   local.get $1
   i32.load offset=8
   i32.store
   i32.const 1808
   return
  end
  local.get $0
  i32.const 1808
  i32.store
  local.get $1
  i32.load offset=4
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/QvModel#hasError (param $0 i32) (result i32)
  local.get $0
  i32.load
  i32.load
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  i32.const 0
  i32.ne
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/QvModel#emit (param $0 i32) (param $1 i32) (param $2 i32)
  (local $3 i32)
  local.get $0
  i32.load
  drop
  i32.const 1
  global.set $~argumentsLength
  local.get $1
  call $~lib/string/String.UTF8.encode@varargs
  local.set $0
  i32.const 1
  global.set $~argumentsLength
  local.get $0
  call $~lib/typedarray/Uint8Array.wrap@varargs
  local.tee $1
  i32.load offset=8
  call $~lib/rt/stub/__alloc
  local.tee $3
  local.get $1
  i32.load offset=4
  local.get $1
  i32.load offset=8
  memory.copy
  i32.const 1
  global.set $~argumentsLength
  local.get $2
  call $~lib/string/String.UTF8.encode@varargs
  local.set $0
  i32.const 1
  global.set $~argumentsLength
  local.get $0
  call $~lib/typedarray/Uint8Array.wrap@varargs
  local.tee $2
  i32.load offset=8
  call $~lib/rt/stub/__alloc
  local.tee $0
  local.get $2
  i32.load offset=4
  local.get $2
  i32.load offset=8
  memory.copy
  local.get $3
  local.get $1
  i32.load offset=8
  local.get $0
  local.get $2
  i32.load offset=8
  call $~lib/qorvum-contract-sdk/assembly/index/__qv_emit_event
 )
 (func $assembly/todo.service/TodoService#create (param $0 i32) (param $1 i32) (result i64)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  (local $8 i32)
  (local $9 i32)
  local.get $1
  i32.const 1888
  call $~lib/qorvum-contract-sdk/assembly/index/getField
  local.set $4
  local.get $1
  i32.const 1392
  call $~lib/qorvum-contract-sdk/assembly/index/getField
  local.set $5
  local.get $1
  i32.const 1552
  call $~lib/qorvum-contract-sdk/assembly/index/getField
  local.set $3
  local.get $1
  i32.const 1600
  call $~lib/qorvum-contract-sdk/assembly/index/getField
  local.set $1
  local.get $4
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  i32.eqz
  if
   i32.const 1984
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  local.get $5
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  i32.eqz
  if
   i32.const 2480
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  call $~lib/qorvum-contract-sdk/assembly/index/Fields#constructor
  i32.const 1392
  local.get $5
  call $~lib/qorvum-contract-sdk/assembly/index/Fields#text
  i32.const 1456
  i32.const 2608
  call $~lib/qorvum-contract-sdk/assembly/index/Fields#text
  i32.const 0
  call $~lib/qorvum-contract-sdk/assembly/index/Fields#bool
  local.set $2
  local.get $3
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  if
   local.get $2
   i32.const 1552
   local.get $3
   call $~lib/qorvum-contract-sdk/assembly/index/Fields#text
   drop
  end
  local.get $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  if
   local.get $2
   i32.const 1600
   local.get $1
   call $~lib/qorvum-contract-sdk/assembly/index/Fields#text
   drop
  end
  block $__inlined_func$~lib/qorvum-contract-sdk/assembly/index/Schema#validate$2 (result i32)
   local.get $0
   i32.load
   i32.load offset=4
   local.set $6
   i32.const 0
   local.set $1
   call $~lib/array/Array<~lib/string/String>#constructor
   local.set $9
   loop $for-loop|0
    local.get $1
    local.get $6
    i32.load offset=8
    i32.load offset=12
    i32.lt_s
    if
     local.get $6
     i32.load offset=8
     local.get $1
     call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#__get
     local.tee $7
     i32.load8_u offset=8
     if (result i32)
      block $__inlined_func$~lib/qorvum-contract-sdk/assembly/index/Fields#has$218 (result i32)
       local.get $7
       i32.load
       local.set $8
       i32.const 0
       local.set $3
       loop $for-loop|00
        local.get $3
        local.get $2
        i32.load
        i32.load offset=12
        i32.lt_s
        if
         i32.const 1
         local.get $2
         i32.load
         local.get $3
         call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#__get
         local.get $8
         call $~lib/string/String.__eq
         br_if $__inlined_func$~lib/qorvum-contract-sdk/assembly/index/Fields#has$218
         drop
         local.get $3
         i32.const 1
         i32.add
         local.set $3
         br $for-loop|00
        end
       end
       i32.const 0
      end
     else
      i32.const 1
     end
     i32.eqz
     if
      local.get $9
      local.get $7
      i32.load
      call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#push
     end
     local.get $1
     i32.const 1
     i32.add
     local.set $1
     br $for-loop|0
    end
   end
   i32.const 1808
   local.get $9
   i32.load offset=12
   i32.eqz
   br_if $__inlined_func$~lib/qorvum-contract-sdk/assembly/index/Schema#validate$2
   drop
   i32.const 2912
   local.get $9
   i32.load offset=4
   local.get $9
   i32.load offset=12
   i32.const 2992
   call $~lib/util/string/joinReferenceArray<~lib/string/String>
   call $~lib/string/String.__concat
  end
  local.tee $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  if
   local.get $1
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  local.get $0
  i32.load
  local.tee $1
  i32.load offset=4
  i32.load offset=4
  local.set $3
  local.get $1
  i32.load
  i32.const 3024
  i32.const 3056
  local.get $1
  i32.load offset=4
  i32.load
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 3104
  call $~lib/string/String.__concat
  local.get $3
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 3152
  call $~lib/string/String.__concat
  local.get $4
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 3184
  call $~lib/string/String.__concat
  local.get $2
  call $~lib/qorvum-contract-sdk/assembly/index/Fields#toJSON
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/Context#_exec
  local.set $1
  local.get $0
  i32.load
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#hasError
  if
   local.get $0
   i32.load
   i32.load
   i32.load
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  local.get $0
  i32.load
  i32.const 3648
  i32.const 3696
  local.get $4
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 3728
  call $~lib/string/String.__concat
  local.get $5
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#emit
  i32.const 3776
  local.get $1
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/writeString
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/QvModel#findById (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  local.get $0
  i32.load offset=4
  i32.load offset=4
  local.set $2
  local.get $0
  i32.load
  i32.const 3888
  i32.const 3056
  local.get $0
  i32.load offset=4
  i32.load
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 3104
  call $~lib/string/String.__concat
  local.get $2
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 3152
  call $~lib/string/String.__concat
  local.get $1
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/Context#_exec
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/QvModel#rel (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  block $__inlined_func$~lib/qorvum-contract-sdk/assembly/index/Schema#_rel$224 (result i32)
   local.get $0
   i32.load offset=4
   local.set $0
   loop $for-loop|0
    local.get $2
    local.get $0
    i32.load offset=12
    i32.load offset=12
    i32.lt_s
    if
     local.get $0
     i32.load offset=12
     local.get $2
     call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#__get
     i32.load
     i32.const 1648
     call $~lib/string/String.__eq
     if
      local.get $0
      i32.load offset=12
      local.get $2
      call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#__get
      br $__inlined_func$~lib/qorvum-contract-sdk/assembly/index/Schema#_rel$224
     end
     local.get $2
     i32.const 1
     i32.add
     local.set $2
     br $for-loop|0
    end
   end
   i32.const 0
  end
  local.tee $0
  if (result i32)
   local.get $0
   i32.load offset=8
  else
   i32.const 1648
  end
  local.set $0
  i32.const 8
  i32.const 17
  call $~lib/rt/stub/__new
  local.tee $2
  i32.const 0
  i32.store
  local.get $2
  i32.const 0
  i32.store offset=4
  local.get $2
  local.get $1
  i32.store
  local.get $2
  local.get $0
  i32.store offset=4
  local.get $2
 )
 (func $assembly/todo.service/TodoService#get (param $0 i32) (param $1 i32) (result i64)
  (local $2 i32)
  local.get $1
  i32.const 1888
  call $~lib/qorvum-contract-sdk/assembly/index/getField
  local.tee $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  i32.eqz
  if
   i32.const 1984
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  local.get $0
  i32.load
  local.get $1
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#findById
  local.set $2
  local.get $0
  i32.load
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#hasError
  if
   i32.const 3920
   local.get $1
   call $~lib/string/String.__concat
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  local.get $2
  i32.const 1552
  call $~lib/qorvum-contract-sdk/assembly/index/getField
  local.tee $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  if
   local.get $0
   i32.load
   local.get $0
   i32.load offset=4
   call $~lib/qorvum-contract-sdk/assembly/index/QvModel#rel
   i32.load
   local.get $1
   call $~lib/qorvum-contract-sdk/assembly/index/QvModel#findById
   local.set $1
   local.get $0
   i32.load offset=4
   call $~lib/qorvum-contract-sdk/assembly/index/QvModel#hasError
   i32.eqz
   if
    i32.const 3776
    i32.const 3984
    local.get $2
    call $~lib/string/String.__concat
    i32.const 4032
    call $~lib/string/String.__concat
    local.get $1
    call $~lib/string/String.__concat
    i32.const 2256
    call $~lib/string/String.__concat
    call $~lib/string/String.__concat
    i32.const 2256
    call $~lib/string/String.__concat
    call $~lib/qorvum-contract-sdk/assembly/index/writeString
    return
   end
  end
  i32.const 3776
  i32.const 3984
  local.get $2
  call $~lib/string/String.__concat
  i32.const 4080
  call $~lib/string/String.__concat
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/writeString
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/QvModel#patch (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  (local $3 i32)
  local.get $0
  i32.load offset=4
  i32.load offset=4
  local.set $3
  local.get $0
  i32.load
  i32.const 4224
  i32.const 3056
  local.get $0
  i32.load offset=4
  i32.load
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 3104
  call $~lib/string/String.__concat
  local.get $3
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 3152
  call $~lib/string/String.__concat
  local.get $1
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 3184
  call $~lib/string/String.__concat
  local.get $2
  call $~lib/qorvum-contract-sdk/assembly/index/Fields#toJSON
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/Context#_exec
 )
 (func $~lib/util/string/strtol<i64> (param $0 i32) (result i64)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i64)
  (local $5 i64)
  (local $6 i32)
  (local $7 i32)
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  local.tee $1
  i32.eqz
  if
   i64.const 0
   return
  end
  local.get $0
  local.tee $2
  i32.load16_u
  local.set $0
  loop $while-continue|0
   block $__inlined_func$~lib/util/string/isSpace$166 (result i32)
    local.get $0
    i32.const 128
    i32.or
    i32.const 160
    i32.eq
    local.get $0
    i32.const 9
    i32.sub
    i32.const 4
    i32.le_u
    i32.or
    local.get $0
    i32.const 5760
    i32.lt_u
    br_if $__inlined_func$~lib/util/string/isSpace$166
    drop
    i32.const 1
    local.get $0
    i32.const -8192
    i32.add
    i32.const 10
    i32.le_u
    br_if $__inlined_func$~lib/util/string/isSpace$166
    drop
    i32.const 1
    local.get $0
    i32.const 5760
    i32.eq
    local.get $0
    i32.const 8232
    i32.eq
    i32.or
    local.get $0
    i32.const 8233
    i32.eq
    local.get $0
    i32.const 8239
    i32.eq
    i32.or
    i32.or
    local.get $0
    i32.const 8287
    i32.eq
    local.get $0
    i32.const 12288
    i32.eq
    i32.or
    local.get $0
    i32.const 65279
    i32.eq
    i32.or
    i32.or
    br_if $__inlined_func$~lib/util/string/isSpace$166
    drop
    i32.const 0
   end
   if
    local.get $2
    i32.const 2
    i32.add
    local.tee $2
    i32.load16_u
    local.set $0
    local.get $1
    i32.const 1
    i32.sub
    local.set $1
    br $while-continue|0
   end
  end
  i64.const 1
  local.set $4
  local.get $0
  i32.const 43
  i32.eq
  local.get $0
  i32.const 45
  i32.eq
  i32.or
  if (result i32)
   local.get $1
   i32.const 1
   i32.sub
   local.tee $1
   i32.eqz
   if
    i64.const 0
    return
   end
   i64.const -1
   i64.const 1
   local.get $0
   i32.const 45
   i32.eq
   select
   local.set $4
   local.get $2
   i32.const 2
   i32.add
   local.tee $2
   i32.load16_u
  else
   local.get $0
  end
  i32.const 48
  i32.eq
  local.get $1
  i32.const 2
  i32.gt_s
  i32.and
  if
   block $break|1
    block $case2|1
     block $case1|1
      local.get $2
      i32.load16_u offset=2
      i32.const 32
      i32.or
      local.tee $0
      i32.const 98
      i32.ne
      if
       local.get $0
       i32.const 111
       i32.eq
       br_if $case1|1
       local.get $0
       i32.const 120
       i32.eq
       br_if $case2|1
       br $break|1
      end
      local.get $2
      i32.const 4
      i32.add
      local.set $2
      local.get $1
      i32.const 2
      i32.sub
      local.set $1
      i32.const 2
      local.set $3
      br $break|1
     end
     local.get $2
     i32.const 4
     i32.add
     local.set $2
     local.get $1
     i32.const 2
     i32.sub
     local.set $1
     i32.const 8
     local.set $3
     br $break|1
    end
    local.get $2
    i32.const 4
    i32.add
    local.set $2
    local.get $1
    i32.const 2
    i32.sub
    local.set $1
    i32.const 16
    local.set $3
   end
  end
  local.get $3
  i32.const 10
  local.get $3
  select
  local.set $3
  local.get $1
  i32.const 1
  i32.sub
  local.set $7
  loop $while-continue|2
   local.get $1
   local.tee $0
   i32.const 1
   i32.sub
   local.set $1
   local.get $0
   if
    block $while-break|2
     local.get $2
     i32.load16_u
     local.tee $6
     i32.const 48
     i32.sub
     local.tee $0
     i32.const 10
     i32.ge_u
     if
      local.get $6
      i32.const 55
      i32.sub
      local.get $6
      i32.const 87
      i32.sub
      local.get $6
      local.get $6
      i32.const 97
      i32.sub
      i32.const 25
      i32.le_u
      select
      local.get $6
      i32.const 65
      i32.sub
      i32.const 25
      i32.le_u
      select
      local.set $0
     end
     local.get $0
     local.get $3
     i32.ge_u
     if
      local.get $1
      local.get $7
      i32.eq
      if
       i64.const 0
       return
      end
      br $while-break|2
     end
     local.get $0
     i64.extend_i32_u
     local.get $5
     local.get $3
     i64.extend_i32_s
     i64.mul
     i64.add
     local.set $5
     local.get $2
     i32.const 2
     i32.add
     local.set $2
     br $while-continue|2
    end
   end
  end
  local.get $4
  local.get $5
  i64.mul
 )
 (func $~lib/util/number/itoa32 (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  local.get $0
  i32.eqz
  if
   i32.const 5312
   return
  end
  i32.const 0
  local.get $0
  i32.sub
  local.get $0
  local.get $0
  i32.const 31
  i32.shr_u
  i32.const 1
  i32.shl
  local.tee $1
  select
  local.tee $0
  i32.const 100000
  i32.lt_u
  if (result i32)
   local.get $0
   i32.const 10
   i32.ge_u
   i32.const 1
   i32.add
   local.get $0
   i32.const 10000
   i32.ge_u
   i32.const 3
   i32.add
   local.get $0
   i32.const 1000
   i32.ge_u
   i32.add
   local.get $0
   i32.const 100
   i32.lt_u
   select
  else
   local.get $0
   i32.const 1000000
   i32.ge_u
   i32.const 6
   i32.add
   local.get $0
   i32.const 1000000000
   i32.ge_u
   i32.const 8
   i32.add
   local.get $0
   i32.const 100000000
   i32.ge_u
   i32.add
   local.get $0
   i32.const 10000000
   i32.lt_u
   select
  end
  local.tee $2
  i32.const 1
  i32.shl
  local.get $1
  i32.add
  i32.const 2
  call $~lib/rt/stub/__new
  local.tee $3
  local.get $1
  i32.add
  local.set $4
  loop $do-loop|0
   local.get $4
   local.get $2
   i32.const 1
   i32.sub
   local.tee $2
   i32.const 1
   i32.shl
   i32.add
   local.get $0
   i32.const 10
   i32.rem_u
   i32.const 48
   i32.add
   i32.store16
   local.get $0
   i32.const 10
   i32.div_u
   local.tee $0
   br_if $do-loop|0
  end
  local.get $1
  if
   local.get $3
   i32.const 45
   i32.store16
  end
  local.get $3
 )
 (func $assembly/todo.service/TodoService#list (param $0 i32) (param $1 i32) (result i64)
  (local $2 i32)
  (local $3 i64)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  local.get $1
  i32.const 1456
  call $~lib/qorvum-contract-sdk/assembly/index/getField
  local.set $4
  block $__inlined_func$~lib/qorvum-contract-sdk/assembly/index/getIntField$227 (result i64)
   i64.const 0
   local.get $1
   i32.const 1920
   i32.const 4560
   call $~lib/string/String.__concat
   i32.const 4592
   call $~lib/string/String.__concat
   local.tee $5
   call $~lib/string/String#indexOf
   local.tee $6
   i32.const 0
   i32.lt_s
   br_if $__inlined_func$~lib/qorvum-contract-sdk/assembly/index/getIntField$227
   drop
   local.get $1
   local.set $2
   local.get $6
   local.get $5
   i32.const 20
   i32.sub
   i32.load offset=16
   i32.const 1
   i32.shr_u
   i32.add
   local.tee $5
   local.set $1
   loop $while-continue|0
    local.get $1
    local.get $2
    i32.const 20
    i32.sub
    i32.load offset=16
    i32.const 1
    i32.shr_u
    i32.lt_s
    if
     local.get $2
     local.get $1
     call $~lib/string/String#charCodeAt
     local.tee $6
     i32.const 48
     i32.lt_s
     local.get $6
     i32.const 57
     i32.gt_s
     i32.or
     i32.eqz
     if
      local.get $1
      i32.const 1
      i32.add
      local.set $1
      br $while-continue|0
     end
    end
   end
   i64.const 0
   local.get $1
   local.get $5
   i32.eq
   br_if $__inlined_func$~lib/qorvum-contract-sdk/assembly/index/getIntField$227
   drop
   local.get $2
   local.get $5
   local.get $1
   call $~lib/string/String#substring
   call $~lib/util/string/strtol<i64>
  end
  local.set $3
  local.get $4
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  if
   i32.const 4624
   i32.const 1456
   call $~lib/qorvum-contract-sdk/assembly/index/escStr
   call $~lib/string/String.__concat
   i32.const 4672
   call $~lib/string/String.__concat
   local.get $4
   call $~lib/qorvum-contract-sdk/assembly/index/escStr
   call $~lib/string/String.__concat
   i32.const 4736
   call $~lib/string/String.__concat
   local.set $2
   i32.const 4
   i32.const 18
   call $~lib/rt/stub/__new
   local.tee $1
   i32.const 0
   i32.store
   local.get $1
   local.get $2
   i32.store
  else
   i32.const 0
   local.set $1
  end
  local.get $0
  i32.load
  local.tee $2
  i32.load
  local.set $4
  local.get $2
  i32.load offset=4
  i32.load
  local.set $5
  local.get $2
  i32.load offset=4
  i32.load offset=4
  local.set $2
  i32.const 24
  i32.const 19
  call $~lib/rt/stub/__new
  local.tee $6
  i32.const 0
  i32.store
  local.get $6
  i32.const 0
  i32.store offset=4
  local.get $6
  i32.const 0
  i32.store offset=8
  local.get $6
  i32.const 3504
  i32.store offset=12
  local.get $6
  i32.const 3504
  i32.store offset=16
  local.get $6
  i32.const 50
  i32.store offset=20
  local.get $6
  local.get $4
  i32.store
  local.get $6
  local.get $5
  i32.store offset=4
  local.get $6
  local.get $2
  i32.store offset=8
  local.get $1
  if
   local.get $6
   local.get $1
   i32.load
   i32.store offset=12
  end
  i32.const 4
  i32.const 20
  call $~lib/rt/stub/__new
  local.tee $1
  i32.eqz
  if
   i32.const 0
   i32.const 0
   call $~lib/rt/stub/__new
   local.set $1
  end
  local.get $1
  call $~lib/array/Array<~lib/string/String>#constructor
  i32.store
  local.get $1
  i32.load
  i32.const 4768
  i32.const 1392
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 4816
  call $~lib/string/String.__concat
  call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#push
  local.get $1
  if
   local.get $6
   i32.const 4880
   local.get $1
   i32.load
   local.tee $1
   i32.load offset=4
   local.get $1
   i32.load offset=12
   i32.const 3296
   call $~lib/util/string/joinReferenceArray<~lib/string/String>
   call $~lib/string/String.__concat
   i32.const 4912
   call $~lib/string/String.__concat
   i32.store offset=16
  end
  local.get $6
  i32.const 50
  local.get $3
  i32.wrap_i64
  local.get $3
  i64.const 0
  i64.le_s
  select
  i32.store offset=20
  local.get $6
  i32.load
  local.set $2
  local.get $6
  i32.load offset=4
  local.set $4
  local.get $6
  i32.load offset=12
  local.set $5
  local.get $6
  i32.load offset=16
  local.set $7
  local.get $6
  i32.load offset=20
  local.set $1
  local.get $6
  i32.load offset=8
  local.tee $6
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  if (result i32)
   i32.const 3104
   local.get $6
   call $~lib/qorvum-contract-sdk/assembly/index/escStr
   call $~lib/string/String.__concat
  else
   i32.const 1808
  end
  local.set $6
  local.get $2
  i32.const 4944
  i32.const 3056
  local.get $4
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  local.get $6
  call $~lib/string/String.__concat
  i32.const 4976
  call $~lib/string/String.__concat
  local.get $5
  i32.const 3504
  local.get $5
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  select
  call $~lib/string/String.__concat
  i32.const 5024
  call $~lib/string/String.__concat
  local.get $7
  i32.const 3504
  local.get $7
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  select
  call $~lib/string/String.__concat
  i32.const 5072
  call $~lib/string/String.__concat
  local.get $1
  call $~lib/util/number/itoa32
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/Context#_exec
  local.set $1
  local.get $0
  i32.load
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#hasError
  if
   local.get $0
   i32.load
   i32.load
   i32.load
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  i32.const 3776
  local.get $1
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/writeString
 )
 (func $assembly/todo.service/TodoService#assign (param $0 i32) (param $1 i32) (result i64)
  (local $2 i32)
  (local $3 i32)
  local.get $0
  i32.load
  i32.load
  drop
  i32.const 1
  global.set $~argumentsLength
  i32.const 5488
  call $~lib/string/String.UTF8.encode@varargs
  local.set $2
  i32.const 1
  global.set $~argumentsLength
  local.get $2
  call $~lib/typedarray/Uint8Array.wrap@varargs
  local.tee $2
  i32.load offset=8
  call $~lib/rt/stub/__alloc
  local.tee $3
  local.get $2
  i32.load offset=4
  local.get $2
  i32.load offset=8
  memory.copy
  local.get $3
  local.get $2
  i32.load offset=8
  call $~lib/qorvum-contract-sdk/assembly/index/__qv_has_role
  i32.const 1
  i32.ne
  if
   i32.const 5536
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  local.get $1
  i32.const 1888
  call $~lib/qorvum-contract-sdk/assembly/index/getField
  local.set $2
  local.get $1
  i32.const 1552
  call $~lib/qorvum-contract-sdk/assembly/index/getField
  local.set $1
  local.get $2
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  i32.eqz
  if
   i32.const 1984
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  local.get $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
  i32.eqz
  if
   i32.const 5600
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  local.get $0
  i32.load
  local.get $2
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#findById
  drop
  local.get $0
  i32.load
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#hasError
  if
   i32.const 3920
   local.get $2
   call $~lib/string/String.__concat
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  local.get $0
  i32.load
  local.get $0
  i32.load offset=4
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#rel
  i32.load
  local.get $1
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#findById
  drop
  local.get $0
  i32.load offset=4
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#hasError
  if
   i32.const 5680
   local.get $1
   call $~lib/string/String.__concat
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  local.get $0
  i32.load
  local.get $2
  call $~lib/qorvum-contract-sdk/assembly/index/Fields#constructor
  i32.const 1552
  local.get $1
  call $~lib/qorvum-contract-sdk/assembly/index/Fields#text
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#patch
  local.set $3
  local.get $0
  i32.load
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#hasError
  if
   local.get $0
   i32.load
   i32.load
   i32.load
   call $~lib/qorvum-contract-sdk/assembly/index/qv_err
   return
  end
  local.get $0
  i32.load
  i32.const 5744
  i32.const 3696
  local.get $2
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 5792
  call $~lib/string/String.__concat
  local.get $1
  call $~lib/qorvum-contract-sdk/assembly/index/escStr
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#emit
  i32.const 3776
  local.get $3
  call $~lib/string/String.__concat
  i32.const 2256
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/writeString
 )
 (func $assembly/index/dispatch (param $0 i32) (param $1 i32) (param $2 i32) (param $3 i32) (result i64)
  (local $4 i32)
  (local $5 i32)
  local.get $0
  local.get $1
  call $~lib/qorvum-contract-sdk/assembly/index/readString
  local.set $1
  local.get $2
  local.get $3
  call $~lib/qorvum-contract-sdk/assembly/index/readString
  local.set $2
  i32.const 4
  i32.const 10
  call $~lib/rt/stub/__new
  local.tee $0
  i32.eqz
  if
   i32.const 0
   i32.const 0
   call $~lib/rt/stub/__new
   local.set $0
  end
  local.get $0
  i32.const 1808
  i32.store
  local.get $0
  global.get $assembly/schema/TodoSchema
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#constructor
  local.set $3
  local.get $0
  global.get $assembly/schema/UserSchema
  call $~lib/qorvum-contract-sdk/assembly/index/QvModel#constructor
  local.set $4
  i32.const 8
  i32.const 11
  call $~lib/rt/stub/__new
  local.tee $0
  i32.const 0
  i32.store
  local.get $0
  i32.const 0
  i32.store offset=4
  local.get $0
  local.get $3
  i32.store
  local.get $0
  local.get $4
  i32.store offset=4
  local.get $1
  i32.const 1840
  call $~lib/string/String.__eq
  if
   local.get $0
   local.get $2
   call $assembly/todo.service/TodoService#create
   return
  end
  local.get $1
  i32.const 3840
  call $~lib/string/String.__eq
  if
   local.get $0
   local.get $2
   call $assembly/todo.service/TodoService#get
   return
  end
  local.get $1
  i32.const 4144
  call $~lib/string/String.__eq
  if
   block $__inlined_func$assembly/todo.service/TodoService#complete$130 (result i64)
    local.get $2
    i32.const 1888
    call $~lib/qorvum-contract-sdk/assembly/index/getField
    local.tee $1
    i32.const 20
    i32.sub
    i32.load offset=16
    i32.const 1
    i32.shr_u
    i32.eqz
    if
     i32.const 1984
     call $~lib/qorvum-contract-sdk/assembly/index/qv_err
     br $__inlined_func$assembly/todo.service/TodoService#complete$130
    end
    local.get $0
    i32.load
    local.get $1
    call $~lib/qorvum-contract-sdk/assembly/index/QvModel#findById
    drop
    local.get $0
    i32.load
    call $~lib/qorvum-contract-sdk/assembly/index/QvModel#hasError
    if
     i32.const 3920
     local.get $1
     call $~lib/string/String.__concat
     call $~lib/qorvum-contract-sdk/assembly/index/qv_err
     br $__inlined_func$assembly/todo.service/TodoService#complete$130
    end
    local.get $0
    i32.load
    local.get $1
    call $~lib/qorvum-contract-sdk/assembly/index/Fields#constructor
    i32.const 1
    call $~lib/qorvum-contract-sdk/assembly/index/Fields#bool
    i32.const 1456
    i32.const 4192
    call $~lib/qorvum-contract-sdk/assembly/index/Fields#text
    call $~lib/qorvum-contract-sdk/assembly/index/QvModel#patch
    local.set $2
    local.get $0
    i32.load
    call $~lib/qorvum-contract-sdk/assembly/index/QvModel#hasError
    if
     local.get $0
     i32.load
     i32.load
     i32.load
     call $~lib/qorvum-contract-sdk/assembly/index/qv_err
     br $__inlined_func$assembly/todo.service/TodoService#complete$130
    end
    local.get $0
    i32.load
    i32.const 4256
    i32.const 3696
    local.get $1
    call $~lib/qorvum-contract-sdk/assembly/index/escStr
    call $~lib/string/String.__concat
    i32.const 2256
    call $~lib/string/String.__concat
    call $~lib/qorvum-contract-sdk/assembly/index/QvModel#emit
    i32.const 3776
    local.get $2
    call $~lib/string/String.__concat
    i32.const 2256
    call $~lib/string/String.__concat
    call $~lib/qorvum-contract-sdk/assembly/index/writeString
   end
   return
  end
  local.get $1
  i32.const 4304
  call $~lib/string/String.__eq
  if
   block $__inlined_func$assembly/todo.service/TodoService#delete$233 (result i64)
    local.get $2
    i32.const 1888
    call $~lib/qorvum-contract-sdk/assembly/index/getField
    local.set $3
    local.get $2
    i32.const 4352
    call $~lib/qorvum-contract-sdk/assembly/index/getField
    local.set $2
    local.get $3
    i32.const 20
    i32.sub
    i32.load offset=16
    i32.const 1
    i32.shr_u
    i32.eqz
    if
     i32.const 1984
     call $~lib/qorvum-contract-sdk/assembly/index/qv_err
     br $__inlined_func$assembly/todo.service/TodoService#delete$233
    end
    local.get $0
    i32.load
    local.tee $4
    i32.load
    local.set $1
    local.get $4
    i32.load offset=4
    i32.load
    local.set $5
    local.get $4
    i32.load offset=4
    i32.load offset=4
    local.set $4
    local.get $2
    i32.const 20
    i32.sub
    i32.load offset=16
    i32.const 1
    i32.shr_u
    if (result i32)
     i32.const 4384
     local.get $2
     call $~lib/qorvum-contract-sdk/assembly/index/escStr
     call $~lib/string/String.__concat
    else
     i32.const 1808
    end
    local.set $2
    local.get $1
    i32.const 4432
    i32.const 3056
    local.get $5
    call $~lib/qorvum-contract-sdk/assembly/index/escStr
    call $~lib/string/String.__concat
    i32.const 3104
    call $~lib/string/String.__concat
    local.get $4
    call $~lib/qorvum-contract-sdk/assembly/index/escStr
    call $~lib/string/String.__concat
    i32.const 3152
    call $~lib/string/String.__concat
    local.get $3
    call $~lib/qorvum-contract-sdk/assembly/index/escStr
    call $~lib/string/String.__concat
    local.get $2
    call $~lib/string/String.__concat
    i32.const 2256
    call $~lib/string/String.__concat
    call $~lib/qorvum-contract-sdk/assembly/index/Context#_exec
    local.set $1
    local.get $0
    i32.load
    call $~lib/qorvum-contract-sdk/assembly/index/QvModel#hasError
    if
     local.get $0
     i32.load
     i32.load
     i32.load
     call $~lib/qorvum-contract-sdk/assembly/index/qv_err
     br $__inlined_func$assembly/todo.service/TodoService#delete$233
    end
    local.get $0
    i32.load
    i32.const 4464
    i32.const 3696
    local.get $3
    call $~lib/qorvum-contract-sdk/assembly/index/escStr
    call $~lib/string/String.__concat
    i32.const 2256
    call $~lib/string/String.__concat
    call $~lib/qorvum-contract-sdk/assembly/index/QvModel#emit
    i32.const 3776
    local.get $1
    call $~lib/string/String.__concat
    i32.const 2256
    call $~lib/string/String.__concat
    call $~lib/qorvum-contract-sdk/assembly/index/writeString
   end
   return
  end
  local.get $1
  i32.const 4512
  call $~lib/string/String.__eq
  if
   local.get $0
   local.get $2
   call $assembly/todo.service/TodoService#list
   return
  end
  local.get $1
  i32.const 5440
  call $~lib/string/String.__eq
  if
   local.get $0
   local.get $2
   call $assembly/todo.service/TodoService#assign
   return
  end
  i32.const 5856
  local.get $1
  call $~lib/string/String.__concat
  call $~lib/qorvum-contract-sdk/assembly/index/qv_err
 )
 (func $~lib/qorvum-contract-sdk/assembly/index/alloc (param $0 i32) (result i32)
  local.get $0
  call $~lib/rt/stub/__alloc
 )
 (func $~start
  (local $0 i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  i32.const 5900
  global.set $~lib/rt/stub/offset
  i32.const 1280
  call $~lib/qorvum-contract-sdk/assembly/index/Schema#constructor
  global.set $assembly/schema/UserSchema
  i32.const 1360
  call $~lib/qorvum-contract-sdk/assembly/index/Schema#constructor
  i32.const 1392
  i32.const 1
  call $~lib/qorvum-contract-sdk/assembly/index/Schema#text
  i32.const 1456
  i32.const 1
  call $~lib/qorvum-contract-sdk/assembly/index/Schema#text
  local.tee $0
  i32.load offset=8
  i32.const 1488
  i32.const 1520
  i32.const 1
  call $~lib/qorvum-contract-sdk/assembly/index/Column#constructor
  call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#push
  local.get $0
  i32.const 1552
  i32.const 0
  call $~lib/qorvum-contract-sdk/assembly/index/Schema#text
  i32.const 1600
  i32.const 0
  call $~lib/qorvum-contract-sdk/assembly/index/Schema#text
  local.tee $3
  i32.load offset=12
  local.set $0
  global.get $assembly/schema/UserSchema
  local.set $1
  i32.const 13
  i32.const 7
  call $~lib/rt/stub/__new
  local.tee $2
  i32.const 0
  i32.store
  local.get $2
  i32.const 0
  i32.store offset=4
  local.get $2
  i32.const 0
  i32.store offset=8
  local.get $2
  i32.const 0
  i32.store8 offset=12
  local.get $2
  i32.const 1648
  i32.store
  local.get $2
  local.get $1
  i32.store offset=4
  local.get $2
  i32.const 1552
  i32.store offset=8
  local.get $2
  i32.const 0
  i32.store8 offset=12
  local.get $0
  local.get $2
  call $~lib/array/Array<~lib/qorvum-contract-sdk/assembly/index/Column>#push
  local.get $3
  global.set $assembly/schema/TodoSchema
 )
)
