(module
 (type (;0;) (func (param i64) (result i64)))
 (export "for_loop" (func $for_loop))
 (func $for_loop (type 0) (param $0 i64) (result i64)
  (local $1 i64)
  (block $label$0
   (br_if $label$0
    (i64.lt_s
     (get_local $0)
     (i64.const 1)
    )
   )
   (set_local $1
    (get_local $0)
   )
   (loop $label$1
    (br_if $label$1
     (i32.eqz
      (i64.eqz
       (tee_local $1
        (i64.add
         (get_local $1)
         (i64.const -1)
        )
       )
      )
     )
    )
   )
  )
  (return
    (i64.add
     (get_local $0)
     (i64.const 42)
    )
  )
 )
)
