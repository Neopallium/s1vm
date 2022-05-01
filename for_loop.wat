(module
 (type (;0;) (func (param i32) (result i32)))
 (export "for_loop" (func $for_loop))
 (func $for_loop (type 0) (param $0 i32) (result i32)
  (local $1 i32)
  (block $label$0
   (br_if $label$0
    (i32.lt_s
     (get_local $0)
     (i32.const 1)
    )
   )
   (set_local $1
    (get_local $0)
   )
   (loop $label$1
    (br_if $label$1
     (tee_local $1
      (i32.add
       (get_local $1)
       (i32.const -1)
      )
     )
    )
   )
  )
  (return
    (i32.add
     (get_local $0)
     (i32.const 42)
    )
  )
 )
)
