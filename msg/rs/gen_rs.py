import os
import yaml

with open(os.path.dirname(__file__) + "/../msg_ids.yaml", 'r') as stream:
    try:
        print('>> Reading yaml ...')
        data = yaml.safe_load(stream)
    except yaml.YAMLError as exc:
        print(exc)

with open(os.path.dirname(__file__) + '/msg.rs', 'w') as fout:
    print('>> Generating msg.rs ...')
    fout.write('#[allow(non_camel_case_types)]\n\n')
    fout.write('#[allow(dead_code)]\n\n')
    fout.write('pub mod msg {\n\n')
    for x,y in data.items():
        # TODO: #define for len(y) == 1
        if type(y) is int:
            fout.write('pub const ' + x + ': usize = ' + str(y) + ';\n')

    fout.write('\n')

    for x,y in data.items():
        if type(y) is list:
            fout.write('#[derive(Primitive)]\n')
            fout.write('pub enum ' + x + ' { \n')
            for yi in y[:-1]:
                k = list(yi)[0];
                fout.write('    ' + k + ' = ' + str(yi[k]) + ',\n')
            k = list(y[-1])[0];
            fout.write('    ' + k + '= ' + str(y[-1][k]) + '\n')
            fout.write('}\n')
    fout.write('}\n')
    print('>> Success!')
