from setuptools import setup
setup(
    name='test',
    version='0.0.1',
    entry_points={
        'console_scripts': [
            'test=test:run'
        ]
    }
)
