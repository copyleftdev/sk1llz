#!/usr/bin/env clojure
;; simplicity_check.clj
;; Analyze Clojure code for complexity according to Rich Hickey's principles.
;;
;; Usage:
;;   clojure simplicity_check.clj path/to/file.clj
;;   clojure simplicity_check.clj --demo

(ns simplicity-check
  (:require [clojure.string :as str]
            [clojure.java.io :as io]))

;; Complexity indicators per Hickey's "Simple Made Easy"
(def complexity-patterns
  [{:name "Mutable State"
    :pattern #"\batom\b|\bref\b|\bagent\b"
    :description "Consider if mutation is truly needed. Can you use pure functions instead?"
    :severity :medium}
   
   {:name "Complex Conditionals"
    :pattern #"\(cond\s+(?:[^\)]+\n){5,}"
    :description "Large cond forms may indicate a need for polymorphism or data-driven design."
    :severity :medium}
   
   {:name "Nested Threading"
    :pattern #"->>?\s+\([^)]*->>?"
    :description "Nested threading macros can be hard to follow. Consider extracting functions."
    :severity :low}
   
   {:name "def Inside defn"
    :pattern #"\(defn[^)]+\n[^)]*\(def\b"
    :description "Definitions inside functions create hidden global state."
    :severity :high}
   
   {:name "Multiple Responsibilities"
    :pattern #"\(defn\s+\S+\s+\[[^\]]*\][^)]*(?:\(if\b[^)]*\){3,}|\(let\s+\[[^\]]*\n[^\]]*\n[^\]]*\n[^\]]*\])"
    :description "Function may be doing too much. Consider splitting."
    :severity :medium}
   
   {:name "String Concatenation"
    :pattern #"\(str\s+[^)]+\s+[^)]+\s+[^)]+\s+[^)]+\)"
    :description "Complex string building - consider format or templating."
    :severity :low}
   
   {:name "Deep Nesting"
    :pattern #"\(\s*\(\s*\(\s*\(\s*\("
    :description "Deep nesting indicates complexity. Extract into named functions."
    :severity :high}
   
   {:name "Long Parameter List"
    :pattern #"\(defn\s+\S+\s+\[[^\]]{50,}\]"
    :description "Many parameters suggest the function does too much or needs a map."
    :severity :medium}
   
   {:name "Complecting with OOP"
    :pattern #"\bdeftype\b|\bdefrecord\b.*\bdefprotocol\b"
    :description "Consider if data + functions would be simpler than types/protocols."
    :severity :low}
   
   {:name "Exception as Control Flow"
    :pattern #"\(try\s+(?:[^)]+\n){10,}"
    :description "Large try blocks may use exceptions for control flow. Consider returning data."
    :severity :medium}])

(defn analyze-line [line line-num patterns]
  (for [pattern patterns
        :when (re-find (:pattern pattern) line)]
    (assoc pattern :line line-num)))

(defn analyze-file [content]
  (let [lines (str/split-lines content)]
    (flatten
     (for [[idx line] (map-indexed vector lines)]
       (analyze-line line (inc idx) complexity-patterns)))))

(defn analyze-whole-file [content]
  ;; Patterns that need full file context
  (for [pattern complexity-patterns
        :when (re-find (:pattern pattern) content)]
    pattern))

(defn count-forms [content]
  {:defn-count (count (re-seq #"\(defn\s" content))
   :let-count (count (re-seq #"\(let\s" content))
   :if-count (count (re-seq #"\(if\s" content))
   :cond-count (count (re-seq #"\(cond\s" content))
   :atom-count (count (re-seq #"\(atom\s" content))
   :loop-count (count (re-seq #"\(loop\s" content))})

(defn simplicity-score [issues form-counts]
  (let [base-score 100
        issue-penalty (reduce + (map #(case (:severity %)
                                        :high 15
                                        :medium 10
                                        :low 5
                                        0)
                                     issues))
        ;; Penalty for complexity indicators
        complexity-penalty (+ (* 2 (:atom-count form-counts))
                             (* 1 (:loop-count form-counts)))]
    (max 0 (- base-score issue-penalty complexity-penalty))))

(defn format-report [filepath issues form-counts]
  (let [score (simplicity-score issues form-counts)]
    (str
     "\n" (str/join (repeat 60 "="))
     "\n SIMPLICITY ANALYSIS: " filepath
     "\n" (str/join (repeat 60 "="))
     "\n"
     "\n## Form Counts"
     "\n  defn:  " (:defn-count form-counts)
     "\n  let:   " (:let-count form-counts)
     "\n  if:    " (:if-count form-counts)
     "\n  cond:  " (:cond-count form-counts)
     "\n  atom:  " (:atom-count form-counts)
     "\n  loop:  " (:loop-count form-counts)
     "\n"
     "\n## Issues Found: " (count issues)
     "\n"
     (if (empty? issues)
       "\n  ✓ No complexity issues detected!"
       (str/join "\n"
                 (for [issue issues]
                   (str "\n  [" (name (:severity issue)) "] " (:name issue)
                        (when (:line issue) (str " (line " (:line issue) ")"))
                        "\n    → " (:description issue)))))
     "\n"
     "\n## Simplicity Score: " score "/100"
     "\n   " (cond
              (>= score 90) "★★★★★ Excellent - Simple and clean"
              (>= score 75) "★★★★☆ Good - Minor complexity"
              (>= score 60) "★★★☆☆ Moderate - Some simplification needed"
              (>= score 40) "★★☆☆☆ Complex - Consider refactoring"
              :else "★☆☆☆☆ Very Complex - Major simplification needed")
     "\n" (str/join (repeat 60 "="))
     "\n")))

(defn demo []
  (let [sample-code "
(ns example.core
  (:require [clojure.string :as str]))

;; Good: Simple pure function
(defn add [a b]
  (+ a b))

;; Warning: Mutable state
(def counter (atom 0))

(defn increment! []
  (swap! counter inc))

;; Warning: Complex conditional
(defn categorize [x]
  (cond
    (< x 0) :negative
    (= x 0) :zero
    (< x 10) :small
    (< x 100) :medium
    (< x 1000) :large
    :else :huge))

;; Warning: Deep nesting
(defn process [data]
  (if (seq data)
    (if (map? (first data))
      (if (contains? (first data) :value)
        (if (number? (:value (first data)))
          (* 2 (:value (first data)))
          0)
        0)
      0)
    0))

;; Good: Data-driven approach
(def size-categories
  [[1000 :huge]
   [100 :large]
   [10 :medium]
   [0 :small]
   [Integer/MIN_VALUE :negative]])

(defn categorize-simple [x]
  (->> size-categories
       (filter #(<= (first %) x))
       first
       second))
"]
    (println "\n=== SIMPLICITY CHECK DEMO ===\n")
    (println "Analyzing sample Clojure code for complexity...")
    (let [issues (analyze-whole-file sample-code)
          form-counts (count-forms sample-code)]
      (println (format-report "demo.clj" issues form-counts)))))

(defn -main [& args]
  (cond
    (or (empty? args) (= (first args) "--help"))
    (println "Usage: clojure simplicity_check.clj <file.clj>
       clojure simplicity_check.clj --demo")
    
    (= (first args) "--demo")
    (demo)
    
    :else
    (let [filepath (first args)]
      (if (.exists (io/file filepath))
        (let [content (slurp filepath)
              issues (analyze-whole-file content)
              form-counts (count-forms content)]
          (println (format-report filepath issues form-counts)))
        (println "File not found:" filepath)))))

;; Run if executed directly
(when (= *file* (System/getProperty "babashka.file"))
  (apply -main *command-line-args*))

;; For REPL usage
(comment
  (demo)
  (-main "src/core.clj"))
