// Takes a load of modules and walk through the full ast. Should be kind enough to tell bugs

use ethers_solc::{
    artifacts::{
        ast::{lowfidelity::Ast, SourceUnit},
        visitor::Visitable,
    },
    ArtifactId, ConfigurableContractArtifact,
};
use std::collections::HashMap;
use std::{cell::RefCell, collections::btree_map::BTreeMap, path::PathBuf, rc::Rc};
// use std::{fs::File, io::BufReader};

use crate::{
    loader::{Information, Loader},
    walker::AllFindings,
};

pub struct Walker {
    artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    loader: Loader,
    source_map: BTreeMap<String, Vec<usize>>,
}

impl Walker {
    pub fn new(
        artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
        loader: Loader,
        source_map: BTreeMap<String, Vec<usize>>,
    ) -> Self {
        Walker {
            artifact,
            loader,
            source_map,
        }
    }

    // For analyzing a syntax tree, we need an AST "walker" — an object to facilitate the traversal of the tree.
    // The ast module offers two walkers:
    // - ast.NodeVisitor (doesn't allow modification to the input tree)
    // - ast.NodeTransformer (allows modification)
    pub fn traverse(&mut self) -> eyre::Result<AllFindings> {
        let mut all_findings: AllFindings = HashMap::new();

        let mut ids: Vec<usize> = Vec::new();

        for (id, art) in &self.artifact {
            let unique_id = id.identifier();

            let mut ast: Ast = art
                .ast
                .as_ref()
                .unwrap_or_else(|| panic!("no ast found for {}", unique_id))
                .clone();

            let mut ast: SourceUnit = ast.to_typed();
            let mut ast: Rc<RefCell<SourceUnit>> = Rc::from(RefCell::from(ast));

            // dedup same sources
            // TODO: is that bug from the ast ?
            let source = ast.borrow_mut();
            if !ids.contains(&source.id) {
                ids.push(source.id);

                let abs_path = id.source.to_str().unwrap().to_string();
                let lines_to_bytes = &self.source_map.get(&abs_path).unwrap()/*.unwrap_or(&Vec::new())*/;

                // let nodes = &ast.nodes;

                let path = PathBuf::from(&source.absolute_path);
                let name = path.file_name().unwrap();
                let name = name.to_os_string().into_string().unwrap();
                // .sol is redundant
                let name = name.strip_suffix(".sol").unwrap();

                let info = Information {
                    name: name.to_string(),
                    version: id.version.clone(),
                };

                self.visit_source(
                    Rc::clone(&ast),
                    lines_to_bytes,
                    info.clone(),
                    &mut all_findings,
                );

                //                 self.loader.0.iter().for_each(|module| {
                //                     // bulk of all findings from each module
                //                     all_findings.entry(module.name.clone()).or_default();
                //                     let findings: &mut Findings = &mut Vec::new();
                //                     self.visit_source(module, nodes, lines_to_bytes, info.clone(), findings);
                //                     all_findings
                //                         .entry(module.name.clone())
                //                         .and_modify(|f| f.append(findings));
                //                 });
            }
        }

        Ok(all_findings)
    }

    pub fn visit_source(
        &self,
        source: Rc<RefCell<SourceUnit>>,
        lines_to_bytes: &[usize],
        info: Information,
        findings: &mut AllFindings,
    ) {
        let source = source.borrow_mut();
        source
            .clone()
            .visit(&mut source.clone())
            .expect("ast traversal failed!");

        let file = info.name.clone();

        // let mut meta_findings: Findings = findings
        //     .iter()
        //     .map(|(module, mod_findings)| {
        //         mod_findings
        //             .iter()
        //             .map(|finding| MetaFinding {
        //                 finding: finding.clone(),
        //                 meta: Meta {
        //                     file: file.clone(),
        //                     line: finding
        //                         .src
        //                         .map(|src| get_line_position(&src, lines_to_bytes) as u32),
        //                 },
        //             })
        //             .collect()
        //     })
        //     .collect();

        // findings.append(&mut meta_findings);
    }

    // pub fn visit_source(
    //     &self,
    //     module: &DynModule,
    //     sources: &[SourceUnitPart],
    //     lines_to_bytes: &[usize],
    //     info: Information,
    //     findings: &mut Findings,
    // ) {
    //     sources.iter().for_each(|source| {
    //         let mod_findings = module.process_source(source, &info);

    //         let file = info.name.clone();

    //         let mut meta_findings: Findings = mod_findings
    //             .into_iter()
    //             .map(|finding| MetaFinding {
    //                 finding: finding.clone(),
    //                 meta: Meta {
    //                     file: file.clone(),
    //                     line: finding
    //                         .src
    //                         .map(|src| get_line_position(&src, lines_to_bytes) as u32),
    //                 },
    //             })
    //             .collect();

    //         findings.append(&mut meta_findings);
    //     });
    // }
}
