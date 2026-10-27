; this is a comment
(def make-adder
  (lambda (x)
    (lambda (y)
      (add x y))))

(def add-five (make-adder 5))

(println (add-five 10)) ; 15

(println (prod :foo 10)) ; (:foo, 10)
