(def foo
	 (add 2 (mul 2 3)))

(def my-add
	 (lambda (a b)
	   (add a b)))

(println ((lambda (x) (add x 1)) 10))
(println (my-add foo 1))
(println (add :foo 1))
