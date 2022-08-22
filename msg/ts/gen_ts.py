import os
import yaml

with open(os.path.dirname(__file__) + '/../msg_ids.yaml', 'r') as stream:
    try:
        print('>> Reading yaml ...')
        data = yaml.safe_load(stream)
    except yaml.YAMLError as exc:
        print(exc)

with open(os.path.dirname(__file__) + '/msg.ts', 'w') as fout:
    print('>> Generating msg.ts ...')
    for x,y in data.items():
        # TODO: #define for len(y) == 1
        if type(y) is int:
            fout.write('export const ' + x + ':number = ' + str(y) + '\n')

    fout.write('\n')

    for x,y in data.items():
        if type(y) is list:
            fout.write('export enum ' + x + ' { \n')
            for yi in y[:]:
                k = list(yi)[0];
                fout.write('    ' + k + '= ' + str(yi[k]) + ',\n')
            fout.write('}\n\n')
    print('>> Success!')
