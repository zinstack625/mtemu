file_list = files(
	'Call.cs',
	'Command.cs',
	'Emulator.cs',
	'Helpers.cs',
	'PortExtender.cs',
	'Statics.cs'
)

comp = find_program('csc')

engine = custom_target(
	'engine',
	output: 'engine.dll',
	input: file_list,
	command: [comp, '-t:library', '@INPUT@', '-out:@OUTPUT@'],
	install: true,
	install_dir: 'subdir',
)

experiments = executable(
	'shim_experiments',
	'shim_experiments.c',
	install: false,
	dependencies: [dependency('mono-2')],
)
