; this is a comment
; (def make-adder
;   (lambda (x)
;     (lambda (y)
;       (add x y))))
;
; (def add-five (make-adder 5))
;
; (println (add-five 10)) ; 15

(def xy (prod :foo 10)) ; (:foo, 10)
(def (prod x y) xy)
(println x)
(println y)
