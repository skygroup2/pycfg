all:
	maturin build -r -j 10
deploy:
	twine upload ./target/wheels/pycfg-0.1.0-cp312-cp312-manylinux_2_34_x86_64.whl --repository-url=http://10.7.0.50:8080/